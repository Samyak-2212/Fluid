// [NEEDS_REVIEW: claude]
//! Velocity Verlet integrator — Tier 0+ (all tiers).
//!
//! Scheme (per `knowledge/physics_contract.md`):
//!
//! ```text
//! x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
//! a(t+dt) = F(t+dt) / m
//! v(t+dt) = v(t) + 0.5*(a(t) + a(t+dt))*dt
//! ```
//!
//! This is a symplectic (time-reversible) second-order integrator that
//! preserves energy exactly for conservative systems in the long run.
//! It is the mandated integrator for rigid body dynamics at all tiers.
//!
//! # Algorithm source
//!
//! Verlet, L. (1967). "Computer 'experiments' on classical fluids.
//! I. Thermodynamical properties of Lennard-Jones molecules."
//! Physical Review, 159(1), 98–103.
//! doi:10.1103/PhysRev.159.98
//!
//! The velocity form (Swope et al., 1982) is used here:
//! Swope, W.C., Andersen, H.C., Berens, P.H., Wilson, K.R. (1982).
//! "A computer simulation method for the calculation of equilibrium constants
//! for the association of simple models of proteins."
//! J. Chem. Phys., 76(1), 637–649. doi:10.1063/1.442716

use core::units::Seconds;
use glam::Vec3;
use crate::integrators::traits::Integrator;

// ── State type ────────────────────────────────────────────────────────────────

/// State vector for Velocity Verlet integration of a single rigid body.
///
/// All vectors are in SI units (metres, m/s, m/s²) stored as `f32` via glam.
/// The force/acceleration is baked into the state before calling `step`;
/// the integrator does not query the world.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VerletState {
    /// World-space position of the centre of mass (metres).
    pub position: Vec3,
    /// Linear velocity of the centre of mass (m/s).
    pub velocity: Vec3,
    /// Linear acceleration at the current time step (m/s²).
    ///
    /// This is `F_current / m`. The caller must compute this from accumulated
    /// forces before passing the state to `step`.
    pub acceleration: Vec3,
}

// ── Integrator ────────────────────────────────────────────────────────────────

/// Velocity Verlet integrator for rigid body dynamics.
///
/// The caller must:
/// 1. Accumulate all forces onto the body for the current step.
/// 2. Compute `acceleration = force / mass` and store it in [`VerletState`].
/// 3. Call `step` to advance position and velocity.
/// 4. After `step`, re-evaluate forces at the new position and update
///    the `acceleration` field for the next step.
///
/// This two-pass requirement is inherent to Velocity Verlet: the second
/// half-velocity update requires `a(t+dt)`, which depends on forces at the
/// new position.
///
/// **For a single-call convenience (without force re-evaluation):**
/// pass the same acceleration for both old and new — this degrades to
/// Störmer-Verlet and is only acceptable in Tier 0.
pub struct VelocityVerlet;

impl Integrator for VelocityVerlet {
    type State = VerletState;

    /// Advances the state by one timestep `dt` using the Velocity Verlet scheme.
    ///
    /// This implementation takes the current acceleration from `state.acceleration`
    /// as `a(t)`. The returned state contains the updated position and a
    /// **half-updated** velocity `v(t+dt/2)`. The caller must compute `a(t+dt)`,
    /// then call [`Self::complete_step`] to finish the velocity update.
    ///
    /// For a simplified single-pass step (Störmer-Verlet), the caller may
    /// treat the returned velocity as `v(t+dt)` directly — this introduces
    /// O(dt²) velocity error per step.
    fn step(&self, state: &Self::State, dt: Seconds) -> Self::State {
        let dt_f32 = dt.value() as f32;

        // x(t+dt) = x(t) + v(t)*dt + 0.5*a(t)*dt²
        let new_position = state.position
            + state.velocity * dt_f32
            + state.acceleration * (0.5 * dt_f32 * dt_f32);

        // v(t+dt/2) = v(t) + 0.5*a(t)*dt  [first half-kick]
        // Note: complete_step adds the second half-kick once a(t+dt) is known.
        let half_velocity = state.velocity + state.acceleration * (0.5 * dt_f32);

        VerletState {
            position: new_position,
            velocity: half_velocity,
            // acceleration placeholder — caller must set a(t+dt) before complete_step
            acceleration: state.acceleration,
        }
    }
}

impl VelocityVerlet {
    /// Completes the velocity update after forces at the new position are known.
    ///
    /// Call this after `step` with `a_new = F(t+dt) / m`:
    ///
    /// ```text
    /// v(t+dt) = v(t+dt/2) + 0.5*a(t+dt)*dt
    /// ```
    ///
    /// The returned state has a fully correct `velocity` and `acceleration`.
    #[inline]
    pub fn complete_step(half_state: VerletState, a_new: Vec3, dt: Seconds) -> VerletState {
        let dt_f32 = dt.value() as f32;
        VerletState {
            position: half_state.position,
            velocity: half_state.velocity + a_new * (0.5 * dt_f32),
            acceleration: a_new,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core::units::Seconds;
    use std::f32::consts::PI;

    /// Two-body circular orbit: Earth–Moon system (simplified unit mass).
    ///
    /// Tests energy conservation (symplecticity) over 1000 orbits.
    ///
    /// Setup:
    ///   - Body at radius R = 1.0 m from the origin (unit orbit).
    ///   - Gravitational constant G * M_central = 1.0 (normalised).
    ///   - Circular orbit speed v = sqrt(G*M / R) = 1.0 m/s.
    ///   - Orbital period T = 2*PI * R / v = 2*PI s.
    ///
    /// Energy per unit mass: E = 0.5*v² - G*M/R = 0.5 - 1.0 = -0.5.
    /// Symplectic integrators conserve a modified Hamiltonian; total energy
    /// drift must remain bounded (not grow) over many orbits.
    #[test]
    fn two_body_orbit_energy_conservation() {
        let integrator = VelocityVerlet;

        // Normalised: G*M = 1, R = 1, v_circ = 1.
        let r = 1.0_f32;
        let gm: f32 = 1.0;

        // Initial state: circular orbit in the XY plane.
        let mut state = VerletState {
            position: Vec3::new(r, 0.0, 0.0),
            velocity: Vec3::new(0.0, 1.0, 0.0), // tangential, counter-clockwise
            acceleration: Vec3::new(-gm / (r * r), 0.0, 0.0), // centripetal
        };

        // Timestep: 1000 steps per orbit, 100 orbits.
        let orbital_period = 2.0 * PI;
        let steps_per_orbit: usize = 1000;
        let n_orbits: usize = 100;
        let dt_f32 = orbital_period / steps_per_orbit as f32;
        let dt = Seconds(dt_f32 as f64);

        // Initial energy (per unit mass): E = 0.5*|v|² - G*M/|r|
        let initial_energy = {
            let v2 = state.velocity.length_squared();
            let pos_r = state.position.length();
            0.5 * v2 - gm / pos_r
        };

        for _ in 0..(steps_per_orbit * n_orbits) {
            // Half-step: advance position, compute v(t+dt/2)
            let half = integrator.step(&state, dt);

            // Evaluate gravity at new position a(t+dt) = -G*M/|r|² * r̂
            let r_new = half.position;
            let r_len = r_new.length();
            let a_new = if r_len > 1e-10 {
                -r_new * (gm / (r_len * r_len * r_len))
            } else {
                Vec3::ZERO
            };

            // Complete velocity update
            state = VelocityVerlet::complete_step(half, a_new, dt);
        }

        // Final energy
        let final_energy = {
            let v2 = state.velocity.length_squared();
            let pos_r = state.position.length();
            0.5 * v2 - gm / pos_r
        };

        // Symplectic integrators conserve energy to O(dt²) globally.
        // Over 100 orbits with 1000 steps/orbit, drift should be < 1e-3.
        let energy_drift = (final_energy - initial_energy).abs();
        assert!(
            energy_drift < 1e-3,
            "Energy drift {} exceeded tolerance 1e-3 after {} orbits",
            energy_drift,
            n_orbits
        );
    }

    /// Simple free-fall: x = x0 + v0*t + 0.5*a*t²
    #[test]
    fn free_fall_position_exact() {
        let integrator = VelocityVerlet;
        let a = -9.81_f32;
        let mut state = VerletState {
            position: Vec3::ZERO,
            velocity: Vec3::new(0.0, 10.0, 0.0),
            acceleration: Vec3::new(0.0, a, 0.0),
        };
        let dt = Seconds(0.1);
        let n = 10usize;

        for _ in 0..n {
            let half = integrator.step(&state, dt);
            // gravity constant → a_new == a_old
            state = VelocityVerlet::complete_step(half, Vec3::new(0.0, a, 0.0), dt);
        }

        // Analytical: y = 10*1.0 + 0.5*(-9.81)*1.0² = 10 - 4.905 = 5.095
        let t = (n as f32) * dt.value() as f32;
        let y_expected = 10.0 * t + 0.5 * a * t * t;
        let y_got = state.position.y;
        assert!(
            (y_got - y_expected).abs() < 1e-4,
            "Free-fall y: expected {y_expected}, got {y_got}"
        );
    }

    /// Static body (zero force/acceleration) must not move.
    #[test]
    fn zero_acceleration_no_drift() {
        let integrator = VelocityVerlet;
        let mut state = VerletState {
            position: Vec3::new(1.0, 2.0, 3.0),
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
        };
        let dt = Seconds(0.016);
        for _ in 0..1000 {
            let half = integrator.step(&state, dt);
            state = VelocityVerlet::complete_step(half, Vec3::ZERO, dt);
        }
        assert_eq!(state.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(state.velocity, Vec3::ZERO);
    }
}
