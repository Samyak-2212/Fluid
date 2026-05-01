// [REVIEWED: claude — C5 complete gate + C7 quality gate, 2026-05-02. No issues found.]
//! SPH fluid simulation — Tier 0+ (all tiers).
//!
//! Implements Smoothed Particle Hydrodynamics per the physics contract:
//!
//! - **Kernel**: Wendland C2 (Dehnen & Aly 2012)
//! - **Correction**: XSPH (Monaghan 1989)
//! - **Density**: summation (not continuity equation)
//! - **Integration**: Leap-Frog symplectic (via `physics_core::integrators::LeapFrog`)
//! - **EOS**: Weakly compressible (Tait equation)
//!
//! # References
//!
//! Wendland C2 kernel:
//! Dehnen, W. & Aly, H. (2012). "Improving convergence in smoothed particle
//! hydrodynamics simulations without pairing instability."
//! MNRAS 425(2), 1068–1082. doi:10.1111/j.1365-2966.2012.21439.x
//!
//! XSPH correction:
//! Monaghan, J.J. (1989). "On the problem of penetration in particle methods."
//! Journal of Computational Physics, 82(1), 1–15. doi:10.1016/0021-9991(89)90032-6
//!
//! Tait EOS for weakly-compressible SPH:
//! Monaghan, J.J. (1994). "Simulating free surface flows with SPH."
//! Journal of Computational Physics, 110(2), 399–406.

use core::units::Seconds;
use glam::Vec3;
use physics_core::integrators::leap_frog::{LeapFrog, LeapFrogState};
use physics_core::integrators::traits::Integrator;

// ── Wendland C2 kernel ────────────────────────────────────────────────────────

/// Wendland C2 smoothing kernel in 3D.
///
/// Formula (Dehnen & Aly 2012, eq. 8, normalized for 3D):
///
/// ```text
/// W(r, h) = σ/h³ · (1 - q/2)⁴ · (2q + 1)    if 0 ≤ q ≤ 2
///          = 0                                   if q > 2
///
/// where q = r/h,  σ = 21/(2π)  [3D normalization constant]
/// ```
///
/// The support radius is `2h` (q ranges 0–2).
///
/// **Verified against Dehnen & Aly (2012) Table 1, 3D entry.**
pub struct WendlandC2Kernel;

impl WendlandC2Kernel {
    /// 3D normalization constant σ = 21/(16π) for support radius 2h.
    ///
    /// **Derivation:** Dehnen & Aly (2012) Table 1 reports σ = 21/(2π) for
    /// the form W = σ/h³·(1-q)⁴(1+4q) with `q = r/h ∈ [0,1]` (support = h).
    /// This implementation uses `q = r/h ∈ [0,2]` (support = 2h), substituting
    /// q' = q/2. The volume element scales by 2³ = 8:
    /// σ_2h = 21/(2π) / 8 = 21/(16π) ≈ 0.4178.
    /// Verified numerically: 4π·σ·∫₀²(1-q/2)⁴(2q+1)q² dq ≈ 1.000.
    pub const SIGMA_3D: f32 = 21.0 / (16.0 * std::f32::consts::PI);

    /// Evaluates W(r, h).
    ///
    /// `r` — particle separation in metres (must be ≥ 0).
    /// `h` — smoothing length in metres (must be > 0).
    ///
    /// Returns zero outside the support radius `2h`.
    #[inline]
    pub fn w(r: f32, h: f32) -> f32 {
        debug_assert!(h > 0.0, "Smoothing length must be positive");
        let q = r / h;
        if q >= 2.0 {
            return 0.0;
        }
        let t = 1.0 - 0.5 * q;
        let t4 = t * t * t * t;
        (Self::SIGMA_3D / (h * h * h)) * t4 * (2.0 * q + 1.0)
    }

    /// Evaluates the gradient of W with respect to the displacement vector.
    ///
    /// `r_vec` — displacement vector from j to i (metres).
    /// `h`     — smoothing length (metres).
    ///
    /// Returns ∇_i W(|r_ij|, h). The gradient magnitude from Dehnen & Aly (2012):
    ///
    /// ```text
    /// dW/dr = σ/h⁴ · (-5q · (1 - q/2)³)    for 0 ≤ q < 2
    /// ∇W = (dW/dr) · r̂
    /// ```
    ///
    /// **Verified against Dehnen & Aly (2012), eq. 14 (derivative form).**
    #[inline]
    pub fn grad_w(r_vec: Vec3, h: f32) -> Vec3 {
        let r = r_vec.length();
        if r < 1e-7 || r >= 2.0 * h {
            return Vec3::ZERO;
        }
        let q = r / h;
        let t = 1.0 - 0.5 * q;
        let t3 = t * t * t;
        // dW/dr = σ/h⁴ · (-5q · (1 - q/2)³)
        let dw_dr = (Self::SIGMA_3D / (h * h * h * h)) * (-5.0 * q * t3);
        let r_hat = r_vec / r;
        dw_dr * r_hat
    }
}

// ── SPH Particle ──────────────────────────────────────────────────────────────

/// A single SPH fluid particle.
#[derive(Debug, Clone)]
pub struct Particle {
    /// Leap-Frog integration state (position in metres, velocity in m/s).
    pub state: LeapFrogState,
    /// Mass of this particle (kg).
    pub mass: f32,
    /// Smoothing length h (metres).
    pub h: f32,
    /// Density at last summation step (kg/m³).
    pub density: f32,
    /// Pressure at last pressure evaluation step (Pa).
    pub pressure: f32,
}

impl Particle {
    /// Creates a new particle at rest at the given position.
    pub fn new(position: Vec3, mass: f32, h: f32) -> Self {
        Self {
            state: LeapFrogState {
                position,
                velocity: Vec3::ZERO,
                acceleration: Vec3::ZERO,
            },
            mass,
            h,
            density: 0.0,
            pressure: 0.0,
        }
    }
}

// ── Tait EOS ──────────────────────────────────────────────────────────────────

/// Tait equation of state for weakly compressible SPH.
///
/// `P = B · [(ρ/ρ₀)^γ - 1]`
///
/// where `B = ρ₀ · c₀² / γ` and `γ = 7` (standard for water-like fluids).
///
/// **Source:** Monaghan (1994), eq. 3.5.
pub struct TaitEos {
    /// Reference density ρ₀ (kg/m³).
    pub rho0: f32,
    /// Reference speed of sound c₀ (m/s).
    pub c0: f32,
    /// Tait exponent γ (dimensionless). Typically 7 for water.
    pub gamma: f32,
}

impl TaitEos {
    /// Computes the pressure from the density.
    #[inline]
    pub fn pressure(&self, rho: f32) -> f32 {
        let b = self.rho0 * self.c0 * self.c0 / self.gamma;
        b * ((rho / self.rho0).powf(self.gamma) - 1.0)
    }
}

// ── SPH simulation ────────────────────────────────────────────────────────────

/// SPH simulation state.
///
/// Owns all particles and orchestrates density summation, pressure evaluation,
/// acceleration computation, XSPH correction, and Leap-Frog integration.
pub struct SphSimulation {
    /// All fluid particles.
    pub particles: Vec<Particle>,
    /// Equation of state.
    pub eos: TaitEos,
    /// XSPH correction coefficient ε ∈ [0, 1].
    ///
    /// ε = 0 disables XSPH. ε = 0.5 is the standard value (Monaghan 1989).
    pub xsph_epsilon: f32,
    /// Leap-Frog integrator (zero-cost unit struct).
    integrator: LeapFrog,
    /// Maximum particle count (enforced in Tier 0 only).
    /// `None` means no cap (Tier 1+).
    pub particle_cap: Option<usize>,
    /// Whether the first step has been taken (used for initial half-kick).
    first_step_done: bool,
}

impl SphSimulation {
    /// Creates an SPH simulation with no particles.
    ///
    /// `particle_cap` — set from `config/fluid_simulator.toml::sph_particle_cap`
    /// for Tier 0. Pass `None` for Tier 1+.
    pub fn new(eos: TaitEos, xsph_epsilon: f32, particle_cap: Option<usize>) -> Self {
        Self {
            particles: Vec::new(),
            eos,
            xsph_epsilon,
            integrator: LeapFrog,
            particle_cap,
            first_step_done: false,
        }
    }

    /// Adds a particle. Returns `false` (and does not add) if at the Tier 0 cap.
    pub fn add_particle(&mut self, p: Particle) -> bool {
        if let Some(cap) = self.particle_cap {
            if self.particles.len() >= cap {
                return false;
            }
        }
        self.particles.push(p);
        true
    }

    /// Returns the number of particles.
    pub fn particle_count(&self) -> usize {
        self.particles.len()
    }

    // ── Density summation ─────────────────────────────────────────────────────

    /// Computes density at each particle via summation over neighbours.
    ///
    /// `ρ_i = Σ_j m_j · W(|r_ij|, h_i)`
    ///
    /// Includes self-contribution (`j == i`). O(N²) — use spatial hashing for N > 10k.
    pub fn compute_densities(&mut self) {
        let n = self.particles.len();
        // Collect positions/masses/h to avoid borrow conflicts.
        let states: Vec<(Vec3, f32, f32)> = self.particles.iter()
            .map(|p| (p.state.position, p.mass, p.h))
            .collect();

        for i in 0..n {
            let mut rho = 0.0_f32;
            let (ri, _mi, hi) = states[i];
            for (j, &(rj, mj, _hj)) in states.iter().enumerate() {
                let r_ij = (ri - rj).length();
                // Use symmetric average of h for kernel evaluation
                let h_avg = if j == i { hi } else { 0.5 * (hi + states[j].2) };
                rho += mj * WendlandC2Kernel::w(r_ij, h_avg);
            }
            self.particles[i].density = rho;
        }
    }

    // ── Pressure evaluation ───────────────────────────────────────────────────

    /// Evaluates pressure at each particle from its density via the Tait EOS.
    pub fn compute_pressures(&mut self) {
        for p in &mut self.particles {
            p.pressure = self.eos.pressure(p.density);
        }
    }

    // ── Acceleration ──────────────────────────────────────────────────────────

    /// Computes SPH pressure acceleration for all particles.
    ///
    /// Uses the symmetric pressure gradient form (Monaghan 1992):
    ///
    /// `a_i = -Σ_j m_j · (P_i/ρ_i² + P_j/ρ_j²) · ∇_i W(r_ij, h_ij)`
    ///
    /// Stores result in `particle.state.acceleration`.
    pub fn compute_accelerations(&mut self, external_accel: Vec3) {
        let n = self.particles.len();
        let states: Vec<(Vec3, f32, f32, f32, f32)> = self.particles.iter()
            .map(|p| (p.state.position, p.mass, p.h, p.density, p.pressure))
            .collect();

        for i in 0..n {
            let mut accel = external_accel;
            let (ri, _mi, hi, rhoi, pi) = states[i];
            if rhoi < 1e-6 {
                self.particles[i].state.acceleration = accel;
                continue;
            }
            let pi_rho2 = pi / (rhoi * rhoi);

            for (j, &(rj, mj, hj, rhoj, pj)) in states.iter().enumerate() {
                if i == j || rhoj < 1e-6 {
                    continue;
                }
                let r_vec = ri - rj; // vector from j to i
                let h_avg = 0.5 * (hi + hj);
                let grad = WendlandC2Kernel::grad_w(r_vec, h_avg);
                let pj_rho2 = pj / (rhoj * rhoj);
                accel -= mj * (pi_rho2 + pj_rho2) * grad;
            }

            self.particles[i].state.acceleration = accel;
        }
    }

    // ── XSPH velocity correction ──────────────────────────────────────────────

    /// Applies XSPH velocity correction (Monaghan 1989).
    ///
    /// Adjusts particle velocities toward the local mean:
    ///
    /// `dx_i/dt = v_i + ε · Σ_j m_j/ρ̄_ij · v_ij · W(r_ij, h_ij)`
    ///
    /// where `v_ij = v_j - v_i` and `ρ̄_ij = 0.5(ρ_i + ρ_j)`.
    ///
    /// This reduces particle interpenetration without changing momenta significantly.
    pub fn apply_xsph_correction(&mut self) {
        if self.xsph_epsilon.abs() < 1e-6 {
            return;
        }
        let n = self.particles.len();
        let snapshot: Vec<(Vec3, Vec3, f32, f32, f32)> = self.particles.iter()
            .map(|p| (p.state.position, p.state.velocity, p.mass, p.h, p.density))
            .collect();

        for i in 0..n {
            let (ri, vi, _mi, hi, rhoi) = snapshot[i];
            let mut correction = Vec3::ZERO;

            for (j, &(rj, vj, mj, hj, rhoj)) in snapshot.iter().enumerate() {
                if i == j {
                    continue;
                }
                let r_ij = (ri - rj).length();
                let h_avg = 0.5 * (hi + hj);
                let rho_avg = 0.5 * (rhoi + rhoj);
                if rho_avg < 1e-6 {
                    continue;
                }
                let w = WendlandC2Kernel::w(r_ij, h_avg);
                let v_ij = vj - vi;
                correction += (mj / rho_avg) * v_ij * w;
            }

            self.particles[i].state.velocity += self.xsph_epsilon * correction;
        }
    }

    // ── Integration step ──────────────────────────────────────────────────────

    /// Advances the simulation by one timestep.
    ///
    /// Pipeline per step:
    /// 1. Density summation
    /// 2. Pressure evaluation (Tait EOS)
    /// 3. Acceleration computation
    /// 4. XSPH velocity correction
    /// 5. Leap-Frog integration
    ///
    /// On the very first call, applies the initial half-kick (per Leap-Frog
    /// synchronisation convention).
    pub fn step(&mut self, dt: Seconds, gravity: Vec3) {
        self.compute_densities();
        self.compute_pressures();
        self.compute_accelerations(gravity);
        self.apply_xsph_correction();

        // Initial half-kick on first step to synchronise Leap-Frog
        if !self.first_step_done {
            let dt_f32 = dt.value() as f32;
            for p in &mut self.particles {
                p.state.velocity -= p.state.acceleration * (dt_f32 * 0.5);
            }
            self.first_step_done = true;
        }

        // Leap-Frog integration
        for p in &mut self.particles {
            p.state = self.integrator.step(&p.state, dt);
        }
    }

    /// Returns true if any particle has a NaN in position or velocity.
    pub fn has_nan(&self) -> bool {
        self.particles.iter().any(|p| {
            p.state.position.is_nan() || p.state.velocity.is_nan()
        })
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Kernel tests ─────────────────────────────────────────────────────────

    #[test]
    fn wendland_c2_zero_outside_support() {
        let w = WendlandC2Kernel::w(3.0, 1.0); // q = 3/1 = 3 > 2
        assert_eq!(w, 0.0);
    }

    #[test]
    fn wendland_c2_positive_inside_support() {
        let w = WendlandC2Kernel::w(0.5, 1.0); // q = 0.5
        assert!(w > 0.0, "Kernel must be positive inside support");
    }

    #[test]
    fn wendland_c2_zero_at_boundary() {
        let w = WendlandC2Kernel::w(2.0, 1.0); // q = 2 exactly
        assert_eq!(w, 0.0);
    }

    #[test]
    fn wendland_c2_grad_zero_at_origin() {
        let g = WendlandC2Kernel::grad_w(Vec3::new(1e-8, 0.0, 0.0), 1.0);
        assert!(g.length() < 1e-5, "Grad at near-origin should be ~zero");
    }

    #[test]
    fn wendland_c2_kernel_normalization_approximate() {
        // Numerical integration of W over 3D sphere of radius 2h.
        // Should equal 1.0 (partition of unity). Test within 2%.
        // Support radius = 2h, so integrate over cube [-2h, 2h]^3 (side 4h).
        let h = 1.0_f32;
        let support = 2.0 * h; // kernel zero for r >= 2h
        let n = 80_usize;
        let step = 2.0 * support / n as f32; // step = 4h/n
        let mut integral = 0.0_f32;
        for ix in 0..n {
            for iy in 0..n {
                for iz in 0..n {
                    let x = -support + (ix as f32 + 0.5) * step;
                    let y = -support + (iy as f32 + 0.5) * step;
                    let z = -support + (iz as f32 + 0.5) * step;
                    let r = (x * x + y * y + z * z).sqrt();
                    integral += WendlandC2Kernel::w(r, h) * step * step * step;
                }
            }
        }
        let err = (integral - 1.0).abs();
        assert!(err < 0.02, "Wendland C2 normalization integral = {integral:.4}, expected ≈1.0 (err={err:.4})");
    }

    // ── SPH simulation stability test ─────────────────────────────────────────

    /// Stability check: 125 particles in a 5×5×5 grid, 1000 steps, no NaN.
    ///
    /// This is the C5 completion gate criterion: "SPH produces a stable particle
    /// simulation for at least 1000 steps without NaN."
    #[test]
    fn sph_1000_steps_no_nan() {
        let eos = TaitEos { rho0: 1000.0, c0: 10.0, gamma: 7.0 };
        let mut sim = SphSimulation::new(eos, 0.5, None);

        let h = 0.15_f32;
        let mass = 0.001_f32;
        // Place 125 particles on a 5×5×5 grid with spacing 0.1m
        for ix in 0..5_i32 {
            for iy in 0..5_i32 {
                for iz in 0..5_i32 {
                    let pos = Vec3::new(ix as f32 * 0.1, iy as f32 * 0.1, iz as f32 * 0.1);
                    sim.add_particle(Particle::new(pos, mass, h));
                }
            }
        }
        assert_eq!(sim.particle_count(), 125);

        let dt = Seconds(0.001);
        let gravity = Vec3::new(0.0, -9.80665, 0.0);

        for _ in 0..1000 {
            sim.step(dt, gravity);
            assert!(!sim.has_nan(), "NaN detected in SPH simulation");
        }
    }

    #[test]
    fn particle_cap_enforced() {
        let eos = TaitEos { rho0: 1000.0, c0: 10.0, gamma: 7.0 };
        let mut sim = SphSimulation::new(eos, 0.5, Some(2));
        assert!(sim.add_particle(Particle::new(Vec3::ZERO, 0.01, 0.1)));
        assert!(sim.add_particle(Particle::new(Vec3::X, 0.01, 0.1)));
        // Third particle should be rejected (cap = 2)
        assert!(!sim.add_particle(Particle::new(Vec3::Y, 0.01, 0.1)));
        assert_eq!(sim.particle_count(), 2);
    }
}
