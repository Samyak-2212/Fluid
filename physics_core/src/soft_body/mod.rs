//! Soft body module — mass-spring + Newmark-β integration (Tier 1+).
//!
//! Full FEM structural soft body is owned by C5 (fem_structural crate).
//! This module provides: mass-spring network, Newmark-β time integration.
//!
//! [IMPLEMENTATION PENDING — Tier A required]

#[cfg(feature = "tier_1")]
pub struct SoftBody;
