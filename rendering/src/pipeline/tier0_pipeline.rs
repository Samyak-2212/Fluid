//! rendering/src/pipeline/tier0_pipeline.rs
//!
//! Tier 0 pipeline — no GPU required.
//! Produces draw calls into a `CpuFramebuffer` via software rasterization.
//!
//! Currently a stub: returns a solid-colour clear pass.
//! Actual rasterizer loop will be implemented in next session.

use crate::tier0::CpuFramebuffer;
use crate::tier0::Pixel;

/// Clear the framebuffer with the given ARGB colour.
/// This is the "render pipeline" for Tier 0 — CPU only.
pub fn clear_pass(fb: &mut CpuFramebuffer, color: Pixel) {
    fb.fill(color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tier0::CpuFramebuffer;

    #[test]
    fn clear_pass_fills_buffer() {
        let mut fb = CpuFramebuffer::new(8, 8);
        clear_pass(&mut fb, 0xFF_12_34_56);
        assert!(fb.pixels.iter().all(|&p| p == 0xFF_12_34_56));
    }
}
