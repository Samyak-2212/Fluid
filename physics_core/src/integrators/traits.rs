//! Integrator trait surface for the Fluid physics core.
//!
//! Every integration scheme in `physics_core` implements [`Integrator`].
//! Concrete implementations live in sibling modules:
//!
//! | Module              | Scheme                  | Mandated for               | Min tier |
//! |---------------------|-------------------------|-----------------------------|----------|
//! | `velocity_verlet`   | Velocity Verlet         | Rigid body dynamics        | 0        |
//! | `leap_frog`         | Leap-Frog (symplectic)  | SPH fluid (consumed by C5) | 0        |
//! | `newmark_beta`      | Implicit Newmark-β      | Soft body / FEM            | 1        |
//! | `rk4`              | Runge-Kutta 4th order   | CFD / thermodynamics       | 1        |
//! | `euler`             | Forward Euler           | Tier 0 simplified only     | 0 only   |
//!
//! **Physics contract:** Euler integration is forbidden on Tiers 1–3.
//! It is gated with `#[cfg(feature = "tier_0")]` and must never be exposed on
//! accuracy paths. See `knowledge/physics_contract.md`.

use core::units::Seconds;

// ── Integrator trait ──────────────────────────────────────────────────────────

/// Common interface for numerical time-integration schemes.
///
/// # Type parameters
///
/// - `State` — the full state vector for one simulation body or system.
///   For rigid bodies this is typically a struct combining position, velocity,
///   and accumulated forces. Integrators must not assume a specific layout.
///
/// # Contract
///
/// 1. `step` must be **pure** with respect to `self` — it reads the current
///    state and returns the next state without side-effects observable outside
///    the returned value.
/// 2. `dt` carries SI dimension [`Seconds`]; callers must not pass raw `f64`.
/// 3. The `Send + Sync` bounds allow integrators to be stored in `Arc<dyn …>`
///    for parallel system execution at Tier 1+.
pub trait Integrator: Send + Sync {
    /// The state type consumed and produced by a single integration step.
    type State: Send;

    /// Advances `state` by one timestep `dt` and returns the updated state.
    ///
    /// The caller is responsible for supplying forces/accelerations baked
    /// into `state` before calling `step`. The integrator itself does not
    /// query the world for forces.
    fn step(&self, state: &Self::State, dt: Seconds) -> Self::State;
}

// ── Derivative provider ───────────────────────────────────────────────────────

/// Supplies the time-derivative of a state vector at a given instant.
///
/// Used by multi-stage schemes (RK4) to evaluate the derivative at
/// intermediate time points without coupling the integrator to a specific
/// force-evaluation strategy.
///
/// Implementors typically compute the net force on each degree of freedom
/// and return the corresponding acceleration or velocity derivative.
pub trait DerivativeProvider: Send + Sync {
    /// The state type this provider works over — must match the corresponding
    /// [`Integrator::State`].
    type State: Send;

    /// Returns `dState/dt` at the given `state`.
    ///
    /// `t` is the current simulation time in SI seconds.
    /// `state` is the current state vector.
    fn derivative(&self, state: &Self::State, t: Seconds) -> Self::State;
}
