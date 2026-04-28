//! rendering/src/tier0/mod.rs
//!
//! Tier 0 — CPU software rasterizer using the `softbuffer` crate.
//!
//! This module is gated behind `#[cfg(feature = "tier_0")]`.
//! It provides a headless pixel buffer that can be written to and then
//! encoded as JPEG/PNG for the IDX HTTP preview.
//!
//! softbuffer 0.4.x requires a display handle and window handle to create
//! a real on-screen buffer.  For the headless preview path (no window),
//! we maintain a plain `Vec<u32>` pixel buffer and bypass softbuffer's
//! window integration entirely — softbuffer is used as a CPU rasterization
//! target when a window is available, and the raw Vec<u32> is used otherwise.

#[cfg(feature = "tier_0")]
pub mod renderer;

#[cfg(feature = "tier_0")]
pub use renderer::SoftbufferRenderer;

/// ARGB pixel: 0xAARRGGBB (softbuffer native format on all platforms).
pub type Pixel = u32;

/// A pure CPU pixel buffer — no GPU required.
///
/// Used by the IDX HTTP preview path and any headless rendering.
pub struct CpuFramebuffer {
    pub width: u32,
    pub height: u32,
    /// Pixels in ARGB 0xAARRGGBB order.
    pub pixels: Vec<Pixel>,
}

impl CpuFramebuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![0xFF_00_00_00u32; (width * height) as usize],
        }
    }

    /// Fill the entire buffer with a solid ARGB colour.
    pub fn fill(&mut self, argb: Pixel) {
        self.pixels.fill(argb);
    }

    /// Write a single pixel (bounds-checked).
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, argb: Pixel) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = argb;
        }
    }

    /// Encode the framebuffer as a JPEG byte vector (quality 85).
    ///
    /// Returns `Err` if the `image` crate fails to encode.
    pub fn to_jpeg(&self) -> Result<Vec<u8>, image::ImageError> {
        use image::{ImageBuffer, Rgb};

        let rgb_pixels: Vec<u8> = self
            .pixels
            .iter()
            .flat_map(|&px| {
                let r = ((px >> 16) & 0xFF) as u8;
                let g = ((px >> 8) & 0xFF) as u8;
                let b = (px & 0xFF) as u8;
                [r, g, b]
            })
            .collect();

        let img: ImageBuffer<Rgb<u8>, Vec<u8>> =
            ImageBuffer::from_raw(self.width, self.height, rgb_pixels)
                .ok_or_else(|| image::ImageError::Parameter(
                    image::error::ParameterError::from_kind(
                        image::error::ParameterErrorKind::DimensionMismatch,
                    ),
                ))?;

        let mut buf = Vec::new();
        let mut cursor = std::io::Cursor::new(&mut buf);
        img.write_to(&mut cursor, image::ImageFormat::Jpeg)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_solid_color() {
        let mut fb = CpuFramebuffer::new(320, 240);
        fb.fill(0xFF_FF_00_00); // solid red
        assert!(fb.pixels.iter().all(|&p| p == 0xFF_FF_00_00));
    }

    #[test]
    fn set_pixel_bounds_check() {
        let mut fb = CpuFramebuffer::new(10, 10);
        fb.set_pixel(9, 9, 0xFF_00_FF_00);   // in-bounds
        fb.set_pixel(10, 10, 0xFF_00_00_FF); // out-of-bounds — must not panic
        assert_eq!(fb.pixels[9 * 10 + 9], 0xFF_00_FF_00);
    }

    #[test]
    fn jpeg_encode_does_not_panic() {
        let mut fb = CpuFramebuffer::new(64, 64);
        fb.fill(0xFF_40_80_C0); // arbitrary colour
        let result = fb.to_jpeg();
        assert!(result.is_ok(), "JPEG encode failed: {:?}", result.err());
        assert!(!result.unwrap().is_empty());
    }
}
