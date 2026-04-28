//! Constraint solver trait surface for the Fluid physics core.
//!
//! The constraint system implements the **sequential impulse** method
//! (Erin Catto, GDC 2006). Each constraint generates corrective impulses
//! applied directly to rigid body velocities until all constraints are
//! satisfied or the iteration budget is exhausted.
//!
//! # Iteration count
//!
//! `constraint_solver_iterations` is loaded from `config/physics_core.toml`.
//! Default: 10. It must not be hardcoded here or in any implementation.
//!
//! # Trait layout
//!
//! - [`Constraint`]       — a single kinematic or contact constraint
//! - [`ConstraintSolver`] — iterates a set of constraints over a body slice

use core::units::Seconds;
use crate::rigid_body::RigidBody;

// ── Single constraint ─────────────────────────────────────────────────────────

/// A single bilateral or unilateral constraint between one or more rigid bodies.
///
/// Implementations include:
/// - Contact constraints (non-penetration)
/// - Joint constraints (ball-and-socket, hinge, slider)
/// - Motor constraints
///
/// # Sequential impulse contract
///
/// `solve` applies **one** corrective impulse step. The solver calls `solve`
/// repeatedly across all constraints until convergence or the iteration limit
/// is reached. Each call must leave `bodies` in a physically valid or
/// closer-to-valid state.
///
/// Implementations must:
/// 1. Compute the constraint violation (position or velocity error).
/// 2. Derive the corrective impulse magnitude via the effective mass formula.
/// 3. Clamp the accumulated impulse to the valid range (for unilateral constraints).
/// 4. Apply equal-and-opposite impulses to the affected bodies.
pub trait Constraint: Send + Sync {
    /// Applies one sequential-impulse iteration to the bodies.
    ///
    /// `bodies` is the full body slice for the simulation step. Implementations
    /// index into this slice using the entity indices stored when the constraint
    /// was constructed.
    ///
    /// `dt` is the current fixed timestep in SI seconds.
    fn solve(&self, bodies: &mut [RigidBody], dt: Seconds);

    /// Returns `true` if the constraint is satisfied within the solver's
    /// convergence tolerance.
    ///
    /// Used by the solver to detect early-exit when all constraints are
    /// simultaneously satisfied before the iteration budget is exhausted.
    fn is_satisfied(&self, bodies: &[RigidBody]) -> bool;
}

// ── Constraint solver ─────────────────────────────────────────────────────────

/// Runs the sequential impulse solver over a full set of constraints.
///
/// # Implementation note
///
/// The canonical loop is:
/// ```text
/// for _ in 0..iterations {
///     for c in constraints {
///         c.solve(bodies, dt);
///     }
///     if constraints.iter().all(|c| c.is_satisfied(bodies)) { break; }
/// }
/// ```
///
/// Implementations may reorder constraints per iteration for stability
/// (warm-starting, shuffle) but must not violate the sequential impulse
/// invariant: each impulse is applied immediately so subsequent constraints
/// in the same iteration see the updated velocities.
pub trait ConstraintSolver: Send + Sync {
    /// Runs the sequential impulse loop.
    ///
    /// # Parameters
    ///
    /// - `constraints` — the constraint set for this step.
    /// - `bodies`      — mutable slice of all rigid bodies in the scene.
    /// - `dt`          — fixed timestep in SI seconds.
    /// - `iterations`  — maximum solver iterations, from `config/physics_core.toml`.
    ///   Must not be hardcoded by the caller.
    fn solve_all(
        &mut self,
        constraints: &[Box<dyn Constraint>],
        bodies: &mut [RigidBody],
        dt: Seconds,
        iterations: usize,
    );
}
