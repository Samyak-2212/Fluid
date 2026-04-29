// [NEEDS_REVIEW: claude]
//! Leap-Frog (Störmer-Verlet) integrator — Tier 0+ (all tiers).
//!
//! Mandated for: SPH fluid simulation (consumed by C5).
//!
//! Scheme (per `knowledge/physics_contract.md`):
//!
//! ```text
//! v(t+dt/2) = v(t-dt/2) + a(t) * dt
//! x(t+dt)   = x(t) + v(t+dt/2) * dt
//! ```
//!
//! The Leap-Frog scheme stores half-integer velocities. Positions and
//! accelerations are synchronised (integer time steps); velocities are
//! offset by half a step. This is equivalent to Velocity Verlet but is
//! more memory-efficient for SPH simulations where only one velocity
//! needs to be stored per particle.
//!
//! # Algorithm source
//!
//! Hockney, R.W., Eastwood, J.W. (1988). *Computer Simulation Using Particles.*
//! Taylor & Francis. Chapter 4 (Leap-Frog scheme).
//!
//! Also: Monaghan, J.J. (1992). "Smoothed Particle Hydrodynamics."
//! Annual Review of Astronomy and Astrophysics, 30, 543–574.
//! doi:10.1146/annurev.aa.30.090192.002551
//!
//! # Integration with C5
//!
//! C5 (Sim Components — SPH) consumes this module via:
//! ```rust,ignore
//! use physics_core::integrators::leap_frog::{LeapFrog, LeapFrogState};
//! use physics_core::integrators::traits::Integrator;
//! ```

use core::units::Seconds;
use glam::Vec3;
use crate::integrators::traits::Integrator;

// ── State type ────────────────────────────────────────────────────────────────

/// State vector for Leap-Frog integration.
///
/// **Important convention:** `velocity` stores `v(t - dt/2)` (the half-step
/// velocity from the *previous* step). On the very first step, initialise
/// `velocity` with `v(0) - 0.5*a(0)*dt` (a half-step kick backward) to
/// synchronise properly.
///
/// For the first step only, you may pass `v(0)` directly — this introduces
/// a one-time O(dt) error in velocity but does not affect long-term energy
/// conservation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LeapFrogState {
    /// World-space position at integer time t (metres).
    pub position: Vec3,
    /// Velocity at half-integer time t - dt/2 (m/s).
    ///
    /// After calling `step`, this field holds `v(t + dt/2)`.
    pub velocity: Vec3,
    /// Acceleration at current integer time t (m/s²).
    ///
    /// Computed from forces as `a = F/m`. The caller must update this
    /// *before* calling `step` for the next iteration.
    pub acceleration: Vec3,
}

// ── Integrator ────────────────────────────────────────────────────────────────

/// Leap-Frog integrator.
///
/// # Usage (SPH pipeline)
///
/// ```rust,ignore
/// // Initial half-kick (synchronisation — first step only)
/// state.velocity -= state.acceleration * (dt / 2.0);
///
/// loop {
///     // Evaluate density + pressure → forces → acceleration
///     state.acceleration = compute_sph_acceleration(&particles);
///     state = integrator.step(&state, dt);
/// }
/// ```
pub struct LeapFrog;

impl Integrator for LeapFrog {
    type State = LeapFrogState;

    /// Advances the state by one timestep using the Leap-Frog scheme.
    ///
    /// Computes:
    /// 1. `v(t+dt/2) = v(t-dt/2) + a(t) * dt`
    /// 2. `x(t+dt)   = x(t) + v(t+dt/2) * dt`
    ///
    /// The returned state has:
    /// - `position` = `x(t+dt)` (updated)
    /// - `velocity` = `v(t+dt/2)` (half-step ahead)
    /// - `acceleration` = `a(t)` (unchanged — caller must recompute from new position)
    fn step(&self, state: &Self::State, dt: Seconds) -> Self::State {
        let dt_f32 = dt.value() as f32;

        // v(t+dt/2) = v(t-dt/2) + a(t) * dt
        let v_half = state.velocity + state.acceleration * dt_f32;

        // x(t+dt) = x(t) + v(t+dt/2) * dt
        let x_new = state.position + v_half * dt_f32;

        LeapFrogState {
            position: x_new,
            velocity: v_half,
            // Caller must update acceleration from forces at x_new
            acceleration: state.acceleration,
        }
    }
}

impl LeapFrog {
    /// Synchronises velocity to integer time for output/energy calculation.
    ///
    /// Leap-Frog stores `v(t+dt/2)`. To get `v(t)` for output or energy
    /// measurement, use:
    ///
    /// ```text
    /// v(t) ≈ 0.5 * (v(t-dt/2) + v(t+dt/2))
    /// ```
    ///
    /// This is called "drift" correction. It does not affect the trajectory.
    ///
    /// `prev_velocity` = `v(t-dt/2)`, `curr_velocity` = `v(t+dt/2)`.
    #[inline]
    pub fn synchronised_velocity(prev_velocity: Vec3, curr_velocity: Vec3) -> Vec3 {
        (prev_velocity + curr_velocity) * 0.5
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core::units::Seconds;
    use std::f32::consts::PI;

    /// Harmonic oscillator: x'' = -ω²x, ω = 1.
    ///
    /// Leap-Frog is symplectic and conserves the modified Hamiltonian.
    /// Energy drift must be bounded over many periods.
    #[test]
    fn harmonic_oscillator_energy_bounded() {
        let integrator = LeapFrog;
        let omega: f32 = 1.0;
        let dt_f32 = 0.01_f32;
        let dt = Seconds(dt_f32 as f64);

        // x(0) = 1, v(0) = 0 → E = 0.5*ω²*x² + 0.5*v² = 0.5
        // Synchronise: v(-dt/2) = v(0) - 0.5*a(0)*dt = 0 - 0.5*(-1)*0.01 = 0.005
        let a0 = -omega * omega * 1.0_f32;
        let mut state = LeapFrogState {
            position: Vec3::new(1.0, 0.0, 0.0),
            velocity: Vec3::new(0.0 - 0.5 * a0 * dt_f32, 0.0, 0.0),
            acceleration: Vec3::new(a0, 0.0, 0.0),
        };

        let n_periods = 100;
        let steps_per_period = (2.0 * PI / dt_f32) as usize;

        // Initial energy (at t = 0): E = 0.5*ω²*x² + 0.5*v_sync²
        // v_sync(0) ≈ 0 (initial condition)
        let initial_energy = 0.5 * omega * omega * 1.0_f32 * 1.0_f32; // ≈ 0.5

        for _ in 0..(n_periods * steps_per_period) {
            state = integrator.step(&state, dt);
            // Recompute acceleration from new position
            state.acceleration = Vec3::new(-omega * omega * state.position.x, 0.0, 0.0);
        }

        // Energy using synchronised velocity (half-step correction is tiny)
        let v_sync = state.velocity; // approximation for test
        let final_energy =
            0.5 * omega * omega * state.position.x * state.position.x
            + 0.5 * v_sync.length_squared();

        let drift = (final_energy - initial_energy).abs();
        assert!(
            drift < 0.01,
            "Harmonic oscillator energy drift {} > 0.01 after {} periods",
            drift,
            n_periods
        );
    }

    /// Free particle: x(t) = x0 + v0*t exactly.
    #[test]
    fn free_particle_linear_motion() {
        let integrator = LeapFrog;
        let dt = Seconds(0.1);
        let v0 = Vec3::new(2.0, 3.0, 1.0);

        // For a free particle, a = 0 and v(−dt/2) = v(0) = v0.
        let mut state = LeapFrogState {
            position: Vec3::ZERO,
            velocity: v0,
            acceleration: Vec3::ZERO,
        };

        let n = 20;
        for _ in 0..n {
            state = integrator.step(&state, dt);
        }

        let t = n as f32 * dt.value() as f32;
        let expected = v0 * t;
        let err = (state.position - expected).length();
        assert!(err < 1e-5, "Free particle position error {err} > 1e-5");
    }
}
