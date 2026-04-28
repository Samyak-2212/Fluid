//! rendering/src/tier0/renderer.rs
//!
//! SoftbufferRenderer — windowed CPU rasterizer for Tier 0.
//!
//! Wraps a `softbuffer::Surface` backed by a `winit` window handle.
//! When no window is available, falls back to `CpuFramebuffer`.

use super::CpuFramebuffer;

/// Tier 0 CPU renderer.
///
/// In tests and headless IDX preview, use `CpuFramebuffer` directly.
/// This struct is the windowed variant — it holds a CpuFramebuffer and
/// provides a render call that fills it with a test pattern.
pub struct SoftbufferRenderer {
    pub framebuffer: CpuFramebuffer,
}

impl SoftbufferRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            framebuffer: CpuFramebuffer::new(width, height),
        }
    }

    /// Render a test frame: horizontal gradient from deep blue to teal.
    /// This demonstrates the Tier 0 path is functional without a GPU.
    pub fn render_test_frame(&mut self) {
        let w = self.framebuffer.width;
        let h = self.framebuffer.height;
        for y in 0..h {
            for x in 0..w {
                let r = 0u8;
                let g = ((x as f32 / w as f32) * 128.0) as u8;
                let b = 128u8 + ((y as f32 / h as f32) * 127.0) as u8;
                let argb = 0xFF_00_00_00u32 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
                self.framebuffer.set_pixel(x, y, argb);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_renders_without_gpu() {
        let mut renderer = SoftbufferRenderer::new(320, 240);
        renderer.render_test_frame();
        // At least some pixels differ from the initial black fill.
        let has_non_black = renderer
            .framebuffer
            .pixels
            .iter()
            .any(|&p| (p & 0x00_FF_FF_FF) != 0);
        assert!(has_non_black, "test frame must produce non-black pixels");
    }
}
