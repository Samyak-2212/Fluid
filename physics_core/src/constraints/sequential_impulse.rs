// [REVIEWED: claude — C4 complete gate + C7 quality gate, 2026-05-02. No issues found.]
//! Sequential impulse constraint solver — Tier 0+.
//!
//! Iteration count from `config/physics_core.toml` key `constraint_solver_iterations`.
//! Default: 10.
//!
//! Applies sequential impulses to satisfy kinematic and dynamic constraints.

use core::units::Seconds;
use crate::rigid_body::RigidBody;
use crate::constraints::traits::{Constraint, ConstraintSolver};

/// Sequential impulse constraint solver.
pub struct SequentialImpulseSolver;

impl ConstraintSolver for SequentialImpulseSolver {
    fn solve_all(
        &mut self,
        constraints: &[Box<dyn Constraint>],
        bodies: &mut [RigidBody],
        dt: Seconds,
        iterations: usize,
    ) {
        if constraints.is_empty() {
            return;
        }

        // Sequential impulse iteration loop
        for _ in 0..iterations {
            let mut all_satisfied = true;

            // Solve each constraint sequentially
            for constraint in constraints {
                constraint.solve(bodies, dt);
                
                // For early exit, we ideally check if *all* are satisfied.
                // However, `is_satisfied` can be expensive, and calling it
                // on every constraint every iteration might be slower than
                // just doing the max iterations or checking once per loop.
                // The trait contract suggests we can break early.
                if !constraint.is_satisfied(bodies) {
                    all_satisfied = false;
                }
            }

            // Early exit if convergence achieved
            if all_satisfied {
                break;
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use core::units::Kilograms;
    use glam::{Mat3, Vec3};

    /// A dummy constraint that forces a body's X velocity to exactly 0.
    struct StopXConstraint {
        body_idx: usize,
    }

    impl Constraint for StopXConstraint {
        fn solve(&self, bodies: &mut [RigidBody], _dt: Seconds) {
            let b = &mut bodies[self.body_idx];
            if !b.is_static {
                let v = b.velocity.x;
                // Apply impulse to cancel v.x
                // Δv = J / m  =>  J = Δv * m
                // We want Δv = -v.x
                let impulse_mag = -v * b.mass.value() as f32;
                let impulse = Vec3::new(impulse_mag, 0.0, 0.0);
                b.apply_impulse(impulse);
            }
        }

        fn is_satisfied(&self, bodies: &[RigidBody]) -> bool {
            bodies[self.body_idx].velocity.x.abs() < 1e-5
        }
    }

    #[test]
    fn sequential_impulse_satisfies_constraint() {
        let mut solver = SequentialImpulseSolver;
        let mut bodies = vec![
            RigidBody::new(Kilograms(2.0), Mat3::IDENTITY),
        ];
        
        // Give body initial velocity
        bodies[0].velocity = Vec3::new(5.0, 2.0, 0.0);
        
        let c = StopXConstraint { body_idx: 0 };
        let constraints: Vec<Box<dyn Constraint>> = vec![Box::new(c)];
        
        solver.solve_all(&constraints, &mut bodies, Seconds(0.016), 10);
        
        assert!(bodies[0].velocity.x.abs() < 1e-5, "Constraint failed to stop X velocity");
        assert_eq!(bodies[0].velocity.y, 2.0, "Y velocity should be unaffected");
    }
}
