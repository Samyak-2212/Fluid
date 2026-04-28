//! rendering/src/surface.rs
//!
//! [NEEDS_REVIEW: claude]
//! wgpu surface and swapchain management.
//! Surface creation requires a valid window handle (raw-window-handle 0.6).
//! Tier 0 does not use this module — it uses the softbuffer path in `tier0/`.

use wgpu::{Surface, SurfaceConfiguration, TextureFormat, PresentMode};
use crate::device::GpuContext;

/// Manages a wgpu surface and swapchain configuration.
pub struct RenderSurface<'window> {
    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
}

impl<'window> RenderSurface<'window> {
    /// Create a surface from a raw window handle.
    ///
    /// `width` / `height` are the initial framebuffer dimensions in pixels.
    pub fn new(
        ctx: &GpuContext,
        target: impl raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle + Send + Sync + 'window,
        width: u32,
        height: u32,
    ) -> Result<Self, SurfaceError> {
        let surface = ctx.instance
            .create_surface(target)
            .map_err(SurfaceError::Create)?;

        let caps = surface.get_capabilities(&ctx.adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&ctx.device, &config);

        Ok(Self { surface, config })
    }

    /// Resize the swapchain. Must be called whenever the window is resized.
    pub fn resize(&mut self, ctx: &GpuContext, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&ctx.device, &self.config);
    }

    /// Returns the surface texture format in use.
    pub fn format(&self) -> TextureFormat {
        self.config.format
    }
}

/// Errors that can occur during surface creation.
#[derive(Debug)]
pub enum SurfaceError {
    Create(wgpu::CreateSurfaceError),
}

impl std::fmt::Display for SurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SurfaceError::Create(e) => write!(f, "surface creation failed: {e}"),
        }
    }
}

impl std::error::Error for SurfaceError {}
