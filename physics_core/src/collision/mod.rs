//! Collision detection module — traits, GJK, EPA, broadphase.
//!
//! # Pipeline
//!
//! Broadphase (spatial hash) → GJK narrowphase → EPA penetration query
//! → ContactManifold → ConstraintSolver

pub mod traits;

// Concrete implementations — stubs; Tier A post-gate work.
pub mod gjk;
pub mod epa;
pub mod broadphase;
