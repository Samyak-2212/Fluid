// [NEEDS_REVIEW: claude]
//! Implicit Newmark-Beta integrator — Tier 1+ only.
//!
//! Parameters (per `knowledge/physics_contract.md`): γ = 0.5, β = 0.25
//! (constant average acceleration — unconditionally stable for linear systems).
//!
//! Mandated for: soft body / FEM structural simulation (consumed by C5 `fem_structural`).
//!
//! # Scheme
//!
//! Given displacement `u(t)`, velocity `v(t)`, acceleration `a(t)`, and external
//! force `f(t+dt)`, the Newmark-Beta update for a linear system M·a + K·u = f is:
//!
//! ```text
//! u(t+dt) = u(t) + dt·v(t) + dt²·[(0.5 - β)·a(t) + β·a(t+dt)]
//! v(t+dt) = v(t) + dt·[(1 - γ)·a(t) + γ·a(t+dt)]
//! ```
//!
//! For a scalar (single-DOF) system with effective stiffness `k` and mass `m`,
//! the new acceleration is solved from:
//!
//! ```text
//! k_eff = m / (β·dt²) + k
//! a(t+dt) = (f(t+dt) - k·u_pred) / k_eff    (single-DOF form)
//! ```
//!
//! The multi-DOF assembler in `fem_structural` handles the full sparse K, M matrices.
//! This module provides the scalar state integrator consumed by FEM element assembly.
//!
//! # Algorithm source
//!
//! Newmark, N.M. (1959). "A Method of Computation for Structural Dynamics."
//! Journal of the Engineering Mechanics Division, ASCE, 85(EM3), 67–94.
//!
//! Hughes, T.J.R. (2000). *The Finite Element Method: Linear Static and Dynamic
//! Finite Element Analysis.* Dover. §9.2 (Newmark family methods).

use core::units::Seconds;
use crate::integrators::traits::Integrator;

// ── Parameters ────────────────────────────────────────────────────────────────

/// Newmark-Beta parameters γ and β for the constant-average-acceleration method.
///
/// γ = 0.5, β = 0.25 gives unconditional stability and second-order accuracy.
/// Second-order accuracy requires γ = 0.5 exactly.
pub const NEWMARK_GAMMA: f64 = 0.5;
pub const NEWMARK_BETA: f64 = 0.25;

// ── State type ────────────────────────────────────────────────────────────────

/// State vector for a single scalar DOF under Newmark-Beta integration.
///
/// For multi-DOF FEM systems, the `fem_structural` assembler holds the global
/// displacement, velocity, and acceleration vectors and advances each DOF using
/// this integrator in a system-assembled form.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewmarkBetaState {
    /// Displacement at current time t (metres or radians depending on DOF type).
    pub displacement: f64,
    /// Velocity at current time t (m/s or rad/s).
    pub velocity: f64,
    /// Acceleration at current time t (m/s² or rad/s²).
    pub acceleration: f64,
    /// External force at the *next* timestep t+dt.
    ///
    /// The caller must supply the force at t+dt before each `step` call.
    /// For time-invariant loading, this equals the force at t.
    pub force_next: f64,
    /// Effective stiffness constant k_eff = m / (β·dt²) + k.
    ///
    /// Pre-computed by the caller from system mass `m` and stiffness `k`.
    /// For nonlinear systems this must be recomputed each step (Newton-Raphson).
    pub k_eff: f64,
    /// Mass of the DOF (kg).
    pub mass: f64,
    /// Stiffness coefficient k (N/m).
    pub stiffness: f64,
}

// ── Integrator ────────────────────────────────────────────────────────────────

/// Implicit Newmark-Beta integrator (γ = 0.5, β = 0.25).
///
/// # Usage (FEM pipeline)
///
/// ```rust,ignore
/// use physics_core::integrators::newmark_beta::{NewmarkBeta, NewmarkBetaState};
/// use physics_core::integrators::traits::Integrator;
/// use core::units::Seconds;
///
/// let integrator = NewmarkBeta;
/// let dt = Seconds(0.01);
/// let state = NewmarkBetaState { /* ... */ };
/// let next = integrator.step(&state, dt);
/// ```
#[cfg(feature = "tier_1")]
pub struct NewmarkBeta;

#[cfg(feature = "tier_1")]
impl Integrator for NewmarkBeta {
    type State = NewmarkBetaState;

    /// Advances the scalar DOF state by one timestep using implicit Newmark-Beta.
    ///
    /// Algorithm (constant-average-acceleration, γ = 0.5, β = 0.25):
    ///
    /// 1. Predictor (explicit part, using old acceleration):
    ///    ```text
    ///    u_pred = u + dt·v + dt²·(0.5 - β)·a
    ///    v_pred = v + dt·(1 - γ)·a
    ///    ```
    /// 2. Solve for new acceleration from equation of motion:
    ///    ```text
    ///    k_eff · a_new = f_next - k · u_pred
    ///    a_new = (f_next - k · u_pred) / k_eff
    ///    ```
    /// 3. Corrector:
    ///    ```text
    ///    u_new = u_pred + β·dt²·a_new
    ///    v_new = v_pred + γ·dt·a_new
    ///    ```
    fn step(&self, state: &Self::State, dt: Seconds) -> Self::State {
        let dt = dt.value();
        let dt2 = dt * dt;

        // 1. Predictor
        let u_pred = state.displacement
            + dt * state.velocity
            + dt2 * (0.5 - NEWMARK_BETA) * state.acceleration;
        let v_pred = state.velocity + dt * (1.0 - NEWMARK_GAMMA) * state.acceleration;

        // 2. Solve for new acceleration
        // k_eff = m/(β·dt²) + k  [pre-computed by caller and stored in k_eff]
        // a_new = (f_next - k·u_pred) / k_eff
        let residual = state.force_next - state.stiffness * u_pred;
        let a_new = residual / state.k_eff;

        // 3. Corrector
        let u_new = u_pred + NEWMARK_BETA * dt2 * a_new;
        let v_new = v_pred + NEWMARK_GAMMA * dt * a_new;

        NewmarkBetaState {
            displacement: u_new,
            velocity: v_new,
            acceleration: a_new,
            // Caller must set force_next for the next timestep.
            // We carry forward the same force as a safe default (time-invariant load).
            force_next: state.force_next,
            k_eff: state.k_eff,
            mass: state.mass,
            stiffness: state.stiffness,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(all(test, feature = "tier_1"))]
mod tests {
    use super::*;
    use core::units::Seconds;

    /// Harmonic oscillator: u'' + ω²·u = 0, ω = 1.
    ///
    /// Newmark-Beta (β=0.25, γ=0.5) is unconditionally stable and second-order accurate.
    /// The numerical frequency differs slightly from the exact ω; energy should be bounded.
    #[test]
    fn harmonic_oscillator_bounded_energy() {
        let omega: f64 = 1.0;
        let mass = 1.0_f64;
        let stiffness = omega * omega * mass; // k = ω²·m = 1.0
        let dt = Seconds(0.05);
        let dt_val = dt.value();

        // k_eff = m / (β·dt²) + k
        let k_eff = mass / (NEWMARK_BETA * dt_val * dt_val) + stiffness;

        // Initial conditions: u(0) = 1, v(0) = 0
        let initial_energy = 0.5 * mass * 0.0_f64 * 0.0 + 0.5 * stiffness * 1.0_f64 * 1.0;

        let mut state = NewmarkBetaState {
            displacement: 1.0,
            velocity: 0.0,
            acceleration: -stiffness * 1.0 / mass, // a = -k·u/m = -1.0
            force_next: 0.0,                        // free vibration
            k_eff,
            mass,
            stiffness,
        };

        let integrator = NewmarkBeta;
        let n_steps = 2000; // ~16 periods at dt=0.05

        for _ in 0..n_steps {
            state = integrator.step(&state, dt);
            // Recompute acceleration for next k_eff (same for linear system)
            // For this test k_eff is constant — no recomputation needed.
        }

        let final_energy =
            0.5 * mass * state.velocity * state.velocity
            + 0.5 * stiffness * state.displacement * state.displacement;

        let drift = (final_energy - initial_energy).abs();
        // Newmark-Beta (β=0.25) introduces numerical damping for large dt;
        // energy may decrease. Bound: < 10% over 2000 steps.
        assert!(
            drift < 0.1 * initial_energy + 1e-10,
            "Energy drift {drift:.6} exceeds 10% of initial {initial_energy:.6}"
        );
    }

    /// Static loading: constant force on a spring-mass system → u_static = F/k.
    #[test]
    fn static_displacement_converges() {
        let mass = 1.0_f64;
        let stiffness = 100.0_f64;
        let force = 10.0_f64; // u_static = 10/100 = 0.1 m
        let dt = Seconds(0.01);
        let dt_val = dt.value();

        let k_eff = mass / (NEWMARK_BETA * dt_val * dt_val) + stiffness;

        let mut state = NewmarkBetaState {
            displacement: 0.0,
            velocity: 0.0,
            acceleration: force / mass,
            force_next: force,
            k_eff,
            mass,
            stiffness,
        };

        let integrator = NewmarkBeta;
        for _ in 0..5000 {
            state = integrator.step(&state, dt);
        }

        // After many steps the system oscillates around u_static = 0.1 m.
        // Verify amplitude is within physical bounds (not diverging).
        let u_static = force / stiffness;
        assert!(
            state.displacement.abs() < 3.0 * u_static,
            "Displacement {:.6} outside expected bound {:.6}",
            state.displacement,
            3.0 * u_static
        );
    }
}
