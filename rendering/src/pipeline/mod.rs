//! rendering/src/pipeline/mod.rs
//!
//! [NEEDS_REVIEW: claude]
//! Render pipeline builders — one per tier.
//!
//! Pipeline layout and render pass structure is Tier A work.
//! This module scaffolds the module tree; concrete pipelines will be
//! implemented in subsequent sessions after wgpu device init is reviewed.

#[cfg(feature = "tier_0")]
pub mod tier0_pipeline;

/// Stub pipeline descriptor shared by all tiers.
///
/// A pipeline wraps a wgpu `RenderPipeline` and knows how to submit a
/// draw call given a vertex buffer and uniforms.
/// Full implementation: Tier A, next session.
pub struct PipelineDescriptor {
    pub label: &'static str,
}
