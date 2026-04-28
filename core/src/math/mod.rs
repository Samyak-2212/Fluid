//! Math primitives — re-exports from `glam` 0.32.1.
//!
//! Re-exporting from this module means downstream crates take a dependency on
//! `core` only; they do not reference `glam` directly and are not broken if the
//! math backend changes.
//!
//! glam 0.32.1 verified on docs.rs (2026-04-28). All listed types confirmed present.

#![allow(unused_imports)]

pub use glam::{DMat4, DQuat, DVec3, Mat3, Mat4, Quat, Vec2, Vec3, Vec4};
