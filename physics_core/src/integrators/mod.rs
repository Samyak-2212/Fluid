//! Integrators module — trait surface and concrete integration schemes.
//!
//! # Modules
//!
//! | Module            | Scheme                | Mandated domain           | Tier |
//! |-------------------|-----------------------|---------------------------|------|
//! | `velocity_verlet` | Velocity Verlet       | Rigid body                | 0+   |
//! | `leap_frog`       | Leap-Frog symplectic  | SPH fluid                 | 0+   |
//! | `newmark_beta`    | Implicit Newmark-β    | Soft body / FEM           | 1+   |
//! | `rk4`             | Runge-Kutta 4         | CFD / thermodynamics      | 1+   |
//! | `euler`           | Forward Euler         | Tier 0 simplified only    | 0    |

pub mod traits;

// Concrete integrators — stubs; implementation is post-gate Tier A work.
// Declare modules now so imports compile; each file contains a stub comment.

pub mod velocity_verlet;
pub mod leap_frog;

#[cfg(feature = "tier_1")]
pub mod newmark_beta;

#[cfg(feature = "tier_1")]
pub mod rk4;

#[cfg(feature = "tier_0")]
pub mod euler;
