//! Motion and force simulator — Tier 0+.
//!
//! Applies forces, torques, and actuator models to [`RigidBody`] entities in
//! the ECS. Does **not** run its own integrator — that is C4's responsibility.
//! This component owns:
//!
//! - Force application: gravity, springs, dampers, motors
//! - Actuator models: hydraulic and electric (configurable)
//! - Joint-driven motion: prismatic, revolute, spherical
//!
//! All quantities use SI units via `core::units`.

use core::units::{Kilograms, Meters, Newtons, Seconds};
use glam::Vec3;
use physics_core::rigid_body::RigidBody;

// ── Gravity ───────────────────────────────────────────────────────────────────

/// Standard gravity vector (m/s²).
pub const GRAVITY_STANDARD: Vec3 = Vec3::new(0.0, -9.80665, 0.0);

/// Applies the gravity force to a rigid body for one timestep.
///
/// `g` is the gravity vector in m/s². Standard Earth gravity:
/// `Vec3::new(0.0, -9.80665, 0.0)`.
///
/// # Contract
/// - Does not apply to static bodies (guarded by `apply_force`).
/// - Force = mass × gravity. The `RigidBody` accumulator is incremented; the
///   integrator zeroes it after consumption.
pub fn apply_gravity(body: &mut RigidBody, g: Vec3) {
    let f = g * body.mass.value() as f32;
    body.apply_force(f);
}

// ── Spring-damper ─────────────────────────────────────────────────────────────

/// Linear spring-damper force between two bodies or between a body and an anchor.
///
/// `F = -k·(|r| - L₀)·r̂ - b·v_rel`
///
/// where `r` is the separation vector, `L₀` the rest length, `k` the stiffness,
/// and `b` the damping coefficient.
#[derive(Debug, Clone, Copy)]
pub struct SpringDamper {
    /// Rest length (m).
    pub rest_length: Meters,
    /// Spring stiffness constant (N/m).
    pub stiffness: f32,
    /// Viscous damping coefficient (N·s/m).
    pub damping: f32,
}

impl SpringDamper {
    /// Computes the spring-damper force on body A given body B's state.
    ///
    /// Returns the force in newtons to be applied to `body_a` (apply negated
    /// force to `body_b` for an equal-and-opposite reaction).
    pub fn force(&self, body_a: &RigidBody, body_b: &RigidBody) -> Vec3 {
        let r = body_a.position - body_b.position;
        let dist = r.length();
        if dist < 1e-6 {
            return Vec3::ZERO;
        }
        let r_hat = r / dist;
        let extension = dist - self.rest_length.value() as f32;
        let spring_f = -self.stiffness * extension * r_hat;
        let v_rel = body_a.velocity - body_b.velocity;
        let damping_f = -self.damping * v_rel.dot(r_hat) * r_hat;
        spring_f + damping_f
    }
}

// ── Actuator models ───────────────────────────────────────────────────────────

/// Hydraulic actuator — applies a force along a local axis.
///
/// Models a double-acting linear hydraulic cylinder.
/// Force direction is the actuator's world-space axis.
#[derive(Debug, Clone)]
pub struct HydraulicActuator {
    /// Maximum force output in newtons.
    pub max_force: Newtons,
    /// Current command signal in [-1.0, 1.0].
    /// Positive: extension force. Negative: retraction force.
    pub command: f32,
    /// World-space axis of actuation (unit vector).
    pub axis: Vec3,
}

impl HydraulicActuator {
    /// Returns the force vector this actuator applies this frame.
    pub fn force_vector(&self) -> Vec3 {
        let magnitude = self.max_force.value() as f32 * self.command.clamp(-1.0, 1.0);
        self.axis * magnitude
    }

    /// Applies the actuator force to the given body at its centre of mass.
    pub fn apply_to(&self, body: &mut RigidBody) {
        body.apply_force(self.force_vector());
    }
}

/// Electric motor — applies a torque about a local axis.
///
/// Models a brushless DC motor with a peak torque limit.
#[derive(Debug, Clone)]
pub struct ElectricMotor {
    /// Peak torque in newton-metres.
    pub peak_torque: f32,
    /// Current command signal in [-1.0, 1.0].
    /// Positive: clockwise (right-hand rule about axis). Negative: CCW.
    pub command: f32,
    /// World-space rotation axis (unit vector).
    pub axis: Vec3,
}

impl ElectricMotor {
    /// Returns the torque vector this motor applies this frame.
    pub fn torque_vector(&self) -> Vec3 {
        let magnitude = self.peak_torque * self.command.clamp(-1.0, 1.0);
        self.axis * magnitude
    }

    /// Applies the motor torque to the given body at its centre of mass.
    pub fn apply_to(&self, body: &mut RigidBody) {
        body.apply_torque(self.torque_vector());
    }
}

// ── Joints ────────────────────────────────────────────────────────────────────

/// Prismatic joint — constrains relative motion to one translation axis.
///
/// This component models the force required to maintain the prismatic
/// constraint. Full constraint resolution is handled by C4's sequential
/// impulse solver; this module provides the joint configuration.
#[derive(Debug, Clone)]
pub struct PrismaticJoint {
    /// World-space translation axis (unit vector).
    pub axis: Vec3,
    /// Lower limit along axis (m).
    pub limit_min: Meters,
    /// Upper limit along axis (m).
    pub limit_max: Meters,
    /// Index of body A in the ECS world.
    pub body_a: usize,
    /// Index of body B in the ECS world.
    pub body_b: usize,
}

/// Revolute joint — one rotational DOF about a fixed axis.
#[derive(Debug, Clone)]
pub struct RevoluteJoint {
    /// World-space rotation axis (unit vector).
    pub axis: Vec3,
    /// Lower angular limit (radians).
    pub limit_min: f32,
    /// Upper angular limit (radians).
    pub limit_max: f32,
    /// Index of body A in the ECS world.
    pub body_a: usize,
    /// Index of body B in the ECS world.
    pub body_b: usize,
}

/// Spherical joint — three rotational DOF, no translation.
#[derive(Debug, Clone)]
pub struct SphericalJoint {
    /// Anchor point in world space (m).
    pub anchor: Vec3,
    /// Index of body A in the ECS world.
    pub body_a: usize,
    /// Index of body B in the ECS world.
    pub body_b: usize,
}

// ── Force accumulation system ─────────────────────────────────────────────────

/// Runs one force-application step over a slice of rigid bodies.
///
/// Applies:
/// 1. Gravity to all non-static bodies.
/// 2. Spring-damper forces between body pairs.
/// 3. Hydraulic actuator forces.
/// 4. Electric motor torques.
///
/// After this call, each body's `force_accum` and `torque_accum` are ready
/// for the C4 integrator step. C4 clears the accumulators after integration.
pub struct ForceSystem {
    /// Gravity vector (m/s²).
    pub gravity: Vec3,
    /// Spring-damper connections: (spring, body_a_idx, body_b_idx).
    pub springs: Vec<(SpringDamper, usize, usize)>,
    /// Hydraulic actuators: (actuator, body_idx).
    pub hydraulics: Vec<(HydraulicActuator, usize)>,
    /// Electric motors: (motor, body_idx).
    pub motors: Vec<(ElectricMotor, usize)>,
}

impl ForceSystem {
    /// Creates a system with standard Earth gravity and no force sources.
    pub fn new() -> Self {
        Self {
            gravity: GRAVITY_STANDARD,
            springs: Vec::new(),
            hydraulics: Vec::new(),
            motors: Vec::new(),
        }
    }

    /// Applies all forces to the given body slice for this timestep.
    ///
    /// `dt` is carried for future viscous/time-dependent forces; currently
    /// unused by the implemented models.
    pub fn apply(&mut self, bodies: &mut [RigidBody], _dt: Seconds) {
        // 1. Gravity
        for body in bodies.iter_mut() {
            apply_gravity(body, self.gravity);
        }

        // 2. Spring-dampers (requires split borrow — we copy state of b first)
        for (spring, ia, ib) in &self.springs {
            if *ia == *ib || *ia >= bodies.len() || *ib >= bodies.len() {
                continue;
            }
            // Safety: ia != ib, so these are disjoint slices.
            let (a_state, b_state) = {
                let a = &bodies[*ia];
                let b = &bodies[*ib];
                (
                    (a.position, a.velocity, a.mass, a.is_static),
                    (b.position, b.velocity, b.mass, b.is_static),
                )
            };
            let body_a_tmp = RigidBody {
                position: a_state.0,
                velocity: a_state.1,
                mass: a_state.2,
                is_static: a_state.3,
                ..RigidBody::new(Kilograms(1.0), glam::Mat3::IDENTITY)
            };
            let body_b_tmp = RigidBody {
                position: b_state.0,
                velocity: b_state.1,
                mass: b_state.2,
                is_static: b_state.3,
                ..RigidBody::new(Kilograms(1.0), glam::Mat3::IDENTITY)
            };
            let f_a = spring.force(&body_a_tmp, &body_b_tmp);
            bodies[*ia].apply_force(f_a);
            bodies[*ib].apply_force(-f_a);
        }

        // 3. Hydraulic actuators
        for (actuator, idx) in &self.hydraulics {
            if *idx < bodies.len() {
                bodies[*idx].apply_force(actuator.force_vector());
            }
        }

        // 4. Electric motors
        for (motor, idx) in &self.motors {
            if *idx < bodies.len() {
                bodies[*idx].apply_torque(motor.torque_vector());
            }
        }
    }
}

impl Default for ForceSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ── Debug overlay ─────────────────────────────────────────────────────────────

/// Diagnostic data for the C6 debugger overlay.
#[cfg(feature = "debug_overlay")]
#[derive(Debug, Default)]
pub struct MotionForceDebugStats {
    /// Number of spring-damper connections evaluated this frame.
    pub spring_evaluations: usize,
    /// Number of actuator forces applied this frame.
    pub actuator_evaluations: usize,
    /// Total magnitude of gravity forces applied (N).
    pub gravity_force_total: f32,
}

// ── Compute FFI traits (Tier 3) ───────────────────────────────────────────────

/// Opaque handle to a GPU compute kernel.
#[cfg(feature = "tier_3")]
pub struct ComputeKernel {
    /// Kernel identifier — implementation-defined.
    pub id: u32,
}

/// Arguments for a GPU compute dispatch.
#[cfg(feature = "tier_3")]
pub struct KernelArgs {
    /// Raw byte buffer passed to the kernel.
    pub data: Vec<u8>,
    /// Number of work-groups to dispatch.
    pub work_groups: [u32; 3],
}

/// GPU compute backend trait — Tier 3 only.
///
/// Isolates all CUDA/ROCm FFI behind this trait. No crate outside C5 may
/// have a direct dependency on CUDA or ROCm.
#[cfg(feature = "tier_3")]
pub trait GpuComputeBackend: Send + Sync {
    /// Dispatches a GPU compute kernel with the given arguments.
    ///
    /// # Errors
    /// Returns an error string if the kernel dispatch fails.
    fn dispatch_kernel(&self, kernel: &ComputeKernel, args: &KernelArgs)
        -> Result<(), Box<dyn std::error::Error>>;
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core::units::{Kilograms, Meters, Seconds};
    use glam::Mat3;

    fn body(mass: f32, pos: Vec3) -> RigidBody {
        let mut b = RigidBody::new(Kilograms(mass as f64), Mat3::IDENTITY);
        b.position = pos;
        b
    }

    #[test]
    fn gravity_applies_correct_force() {
        let mut b = body(2.0, Vec3::ZERO);
        apply_gravity(&mut b, GRAVITY_STANDARD);
        // F = m * g ≈ 2 * 9.80665 ≈ 19.613 N downward
        assert!((b.force_accum.y + 19.613_f32).abs() < 0.01);
    }

    #[test]
    fn static_body_ignores_gravity() {
        let mut b = body(2.0, Vec3::ZERO);
        b.is_static = true;
        apply_gravity(&mut b, GRAVITY_STANDARD);
        assert_eq!(b.force_accum, Vec3::ZERO);
    }

    #[test]
    fn spring_damper_zero_at_rest_length() {
        let spring = SpringDamper {
            rest_length: Meters(1.0),
            stiffness: 10.0,
            damping: 0.0,
        };
        let a = body(1.0, Vec3::new(1.0, 0.0, 0.0));
        let b = body(1.0, Vec3::ZERO);
        let f = spring.force(&a, &b);
        // Distance is exactly rest_length — force should be ~zero
        assert!(f.length() < 1e-5, "Spring force at rest length: {f:?}");
    }

    #[test]
    fn spring_damper_attractive_when_extended() {
        let spring = SpringDamper {
            rest_length: Meters(1.0),
            stiffness: 10.0,
            damping: 0.0,
        };
        let a = body(1.0, Vec3::new(2.0, 0.0, 0.0)); // 1m beyond rest
        let b = body(1.0, Vec3::ZERO);
        let f = spring.force(&a, &b);
        // Force on a should be toward b (negative x)
        assert!(f.x < -1.0, "Spring force on extended body should be negative x: {f:?}");
    }

    #[test]
    fn hydraulic_actuator_zero_at_zero_command() {
        let act = HydraulicActuator {
            max_force: Newtons(1000.0),
            command: 0.0,
            axis: Vec3::X,
        };
        assert_eq!(act.force_vector(), Vec3::ZERO);
    }

    #[test]
    fn electric_motor_torque_direction() {
        let motor = ElectricMotor {
            peak_torque: 100.0,
            command: 1.0,
            axis: Vec3::Y,
        };
        let t = motor.torque_vector();
        assert!((t.y - 100.0).abs() < 1e-4);
    }

    #[test]
    fn force_system_applies_gravity_to_all_dynamic_bodies() {
        let mut bodies = vec![body(1.0, Vec3::ZERO), body(2.0, Vec3::X)];
        bodies[1].is_static = true;
        let mut sys = ForceSystem::new();
        sys.apply(&mut bodies, Seconds(0.016));
        // body[0] should have gravity applied
        assert!(bodies[0].force_accum.y < -9.0);
        // body[1] is static — no force
        assert_eq!(bodies[1].force_accum, Vec3::ZERO);
    }
}
