//! `rendering` crate — wgpu abstraction, Tier 0 CPU rasterizer, scene renderer,
//! and IDX HTTP preview server for the Fluid framework.
//!
//! # Module layout
//!
//! - [`camera`]          — Camera with SI Meters position, view/projection matrices
//! - [`device`]          — wgpu device/adapter/queue initialisation
//! - [`surface`]         — wgpu surface and swapchain
//! - [`pipeline`]        — per-tier render pipeline builders
//! - [`tier0`]           — softbuffer CPU rasterizer (Tier 0, `cfg(feature = "tier_0")`)
//! - [`scene_renderer`]  — `SceneRenderer` trait + `StubRenderer`
//! - [`debug_overlay`]   — frame stats overlay (debug builds)
//! - [`http_preview`]    — IDX browser preview HTTP server on port 8080
//!
//! # Tier strategy
//!
//! Tier is selected at compile time via `FLUID_TIER` env var (see `build.rs`).
//! - Tier 0: CPU only, softbuffer rasterizer, no GPU.
//! - Tier 1–2: wgpu (OpenGL/Vulkan/DX12/Metal backend auto-selected).
//! - Tier 3: wgpu compute + CUDA/ROCm FFI (owned by C5).
//!
//! Do NOT implement manual backend selection.

pub mod camera;
pub mod debug_overlay;
pub mod device;
pub mod http_preview;
pub mod pipeline;
pub mod scene_renderer;
pub mod surface;

#[cfg(feature = "tier_0")]
pub mod tier0;

// Re-export the most commonly used types at crate root.
pub use camera::Camera;
pub use debug_overlay::{DebugOverlay, FrameStats};
pub use http_preview::{publish_frame, start_preview_server, SharedFrame};
pub use scene_renderer::{SceneRenderer, StubRenderer};
