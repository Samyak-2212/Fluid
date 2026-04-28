//! Rigid body component definition.
//!
//! [`RigidBody`] is the central data type for rigid body simulation.
//! It is consumed by:
//! - the `VelocityVerlet` integrator (position / velocity update)
//! - the `ConstraintSolver` (impulse application)
//! - the `CollisionDetector` (shape query via `ConvexShape`)
//!
//! # Unit conventions
//!
//! `glam::Vec3` is unitless — comments on each field document the physical
//! unit. Use `core::units` types at API boundaries (function parameters and
//! return values). Use `glam::Vec3` for internal vector math.
//!
//! # Static bodies
//!
//! When `is_static == true` the integrator must skip the body (infinite mass).
//! The constraint solver must not apply velocity corrections to static bodies.

use core::units::{Kilograms, Newtons};
use glam::{Mat3, Quat, Vec3};

// ── RigidBody ─────────────────────────────────────────────────────────────────

/// Full kinematic and inertial state of a single rigid body.
///
/// Fields that hold `glam` types use SI units by convention (metres, kg, N)
/// but carry no compile-time unit wrapper — the comments are the contract.
/// Fields that hold `core::units` types carry the unit at compile time.
#[derive(Debug, Clone)]
pub struct RigidBody {
    /// World-space position of the centre of mass, in metres.
    pub position: Vec3,

    /// Linear velocity of the centre of mass, in metres per second.
    pub velocity: Vec3,

    /// Orientation as a unit quaternion (no physical unit).
    pub orientation: Quat,

    /// Angular velocity in the world frame, in radians per second.
    pub ang_velocity: Vec3,

    /// Total mass of the body in SI kilograms.
    pub mass: Kilograms,

    /// World-space inertia tensor (kg·m²).
    ///
    /// Updated each frame from the body-space inertia tensor and current
    /// orientation:  `I_world = R * I_body * R^T`.
    pub inertia: Mat3,

    /// Net force accumulated this simulation step, in SI newtons.
    ///
    /// Cleared to zero at the start of each step after the integrator
    /// has consumed it.
    pub force_accum: Vec3,

    /// Net torque accumulated this simulation step, in SI newton-metres.
    ///
    /// Cleared to zero at the start of each step after the integrator
    /// has consumed it.
    pub torque_accum: Vec3,

    /// When `true`, the integrator skips this body (infinite effective mass).
    /// The constraint solver must also skip velocity corrections to static bodies.
    pub is_static: bool,
}

impl RigidBody {
    /// Creates a new rigid body at rest at the origin with the given mass.
    ///
    /// `inertia` should be supplied as the body-space inertia tensor; callers
    /// must rotate it to world space before passing it to the integrator.
    pub fn new(mass: Kilograms, inertia: Mat3) -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            orientation: Quat::IDENTITY,
            ang_velocity: Vec3::ZERO,
            mass,
            inertia,
            force_accum: Vec3::ZERO,
            torque_accum: Vec3::ZERO,
            is_static: false,
        }
    }

    /// Applies a force (in newtons, as raw `Vec3` per the glam convention)
    /// at the body's centre of mass.
    #[inline]
    pub fn apply_force(&mut self, force: Vec3) {
        if !self.is_static {
            self.force_accum += force;
        }
    }

    /// Applies a force at a world-space point, also generating a torque.
    ///
    /// `force` is in newtons (Vec3). `point` is in metres (Vec3).
    #[inline]
    pub fn apply_force_at_point(&mut self, force: Vec3, point: Vec3) {
        if !self.is_static {
            self.force_accum += force;
            let r = point - self.position;
            self.torque_accum += r.cross(force);
        }
    }

    /// Applies a direct torque (newton-metres) at the centre of mass.
    #[inline]
    pub fn apply_torque(&mut self, torque: Vec3) {
        if !self.is_static {
            self.torque_accum += torque;
        }
    }

    /// Resets force and torque accumulators to zero.
    ///
    /// Must be called at the **end** of each integration step, after forces
    /// have been consumed.
    #[inline]
    pub fn clear_accumulators(&mut self) {
        self.force_accum = Vec3::ZERO;
        self.torque_accum = Vec3::ZERO;
    }

    /// Returns the inverse mass, or `0.0` for static bodies (infinite mass).
    #[inline]
    pub fn inv_mass(&self) -> f64 {
        if self.is_static || self.mass.value() == 0.0 {
            0.0
        } else {
            1.0 / self.mass.value()
        }
    }

    /// Applies a linear impulse directly to velocity.
    ///
    /// `impulse` is in newton-seconds (kg·m/s). No SI wrapper used here
    /// because `glam::Vec3` is unitless; the caller is responsible for
    /// correct units.
    #[inline]
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if !self.is_static {
            // Δv = J / m
            self.velocity += impulse * self.inv_mass() as f32;
        }
    }
}

// ── Helper: newtons helper ────────────────────────────────────────────────────

/// Converts a [`Newtons`] SI value to the raw `f64` magnitude.
///
/// Provided as a convenience for callers that build a force vector
/// from a scalar magnitude and a direction.
#[inline]
pub fn newtons_to_f64(n: Newtons) -> f64 {
    n.value()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core::units::Kilograms;

    fn unit_body() -> RigidBody {
        RigidBody::new(Kilograms(1.0), Mat3::IDENTITY)
    }

    #[test]
    fn new_body_at_rest() {
        let b = unit_body();
        assert_eq!(b.position, Vec3::ZERO);
        assert_eq!(b.velocity, Vec3::ZERO);
        assert!(!b.is_static);
    }

    #[test]
    fn apply_force_accumulates() {
        let mut b = unit_body();
        b.apply_force(Vec3::new(1.0, 0.0, 0.0));
        b.apply_force(Vec3::new(0.0, 2.0, 0.0));
        assert_eq!(b.force_accum, Vec3::new(1.0, 2.0, 0.0));
    }

    #[test]
    fn clear_accumulators_zeroes_force_and_torque() {
        let mut b = unit_body();
        b.apply_force(Vec3::X);
        b.apply_torque(Vec3::Y);
        b.clear_accumulators();
        assert_eq!(b.force_accum, Vec3::ZERO);
        assert_eq!(b.torque_accum, Vec3::ZERO);
    }

    #[test]
    fn static_body_ignores_force() {
        let mut b = unit_body();
        b.is_static = true;
        b.apply_force(Vec3::new(99.0, 0.0, 0.0));
        assert_eq!(b.force_accum, Vec3::ZERO);
    }

    #[test]
    fn inv_mass_static_is_zero() {
        let mut b = unit_body();
        b.is_static = true;
        assert_eq!(b.inv_mass(), 0.0);
    }

    #[test]
    fn inv_mass_dynamic() {
        let b = RigidBody::new(Kilograms(2.0), Mat3::IDENTITY);
        assert!((b.inv_mass() - 0.5).abs() < 1e-12);
    }
}
