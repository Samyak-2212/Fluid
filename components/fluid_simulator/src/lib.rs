//! Fluid simulator — SPH (Tier 0+) and CFD (Tier 1+).
//!
//! # Modules
//!
//! - [`sph`] — Smoothed Particle Hydrodynamics (Wendland C2 kernel + XSPH + Leap-Frog)
//! - [`cfd`] — Grid-based CFD (projection method, incompressible Navier-Stokes)
//! - [`compute`] — Tier 3 GPU compute backend FFI trait

pub mod sph;

#[cfg(feature = "tier_1")]
pub mod cfd;

#[cfg(feature = "tier_3")]
pub mod compute;
