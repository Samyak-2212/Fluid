//! `physics_core` crate — physics engine for the Fluid framework.
//!
//! # Module layout
//!
//! | Module           | Contents                                             | Min tier |
//! |------------------|------------------------------------------------------|----------|
//! | [`integrators`]  | Integrator trait + concrete schemes                  | 0        |
//! | [`collision`]    | GJK, EPA, broadphase, collision traits               | 0        |
//! | [`constraints`]  | Sequential impulse constraint solver traits          | 0        |
//! | [`rigid_body`]   | [`RigidBody`] component struct                       | 0        |
//! | [`soft_body`]    | Mass-spring + Newmark-β soft body (Tier 1+)          | 1        |
//!
//! # Physics contract
//!
//! Every integrator choice and numerical method is mandated by
//! `knowledge/physics_contract.md`. Read it before adding any integrator.
//! Euler integration is **forbidden** at Tiers 1–3.
//!
//! # Unit convention
//!
//! All quantities crossing public API boundaries use SI newtype wrappers
//! from [`core::units`]. `glam::Vec3` fields inside structs are unitless by
//! glam convention — field comments document the physical unit.

pub mod collision;
pub mod constraints;
pub mod integrators;
pub mod rigid_body;

#[cfg(feature = "tier_1")]
pub mod soft_body;
