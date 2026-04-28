//! rendering/src/debug_overlay.rs
//!
//! Debug overlay — renders frame stats on top of the scene.
//!
//! Visible only in debug builds (`cfg(debug_assertions)`) or when the
//! `debug_overlay` feature is enabled.  Compiled out entirely in release
//! builds without the feature.

/// Per-frame statistics passed to the overlay.
#[derive(Debug, Clone, Default)]
pub struct FrameStats {
    /// Frame time in milliseconds.
    pub frame_ms: f64,
    /// Frames per second (derived from frame_ms).
    pub fps: f64,
    /// Number of physics steps taken this frame.
    pub physics_step_count: u32,
    /// Number of live entities in the ECS world.
    pub entity_count: usize,
}

impl FrameStats {
    /// Derive FPS from frame_ms.
    pub fn from_frame_ms(frame_ms: f64, physics_steps: u32, entities: usize) -> Self {
        let fps = if frame_ms > 0.0 { 1000.0 / frame_ms } else { 0.0 };
        Self {
            frame_ms,
            fps,
            physics_step_count: physics_steps,
            entity_count: entities,
        }
    }

    /// Format the overlay as a single-line ASCII string.
    pub fn to_display_string(&self) -> String {
        format!(
            "FPS: {:.1} | Frame: {:.2}ms | Physics steps: {} | Entities: {}",
            self.fps, self.frame_ms, self.physics_step_count, self.entity_count
        )
    }
}

/// Debug overlay renderer.
///
/// In the CPU (Tier 0) path, the overlay is written into the `CpuFramebuffer`
/// as ASCII text rows drawn pixel-by-pixel using a minimal 5×7 font.
/// In the wgpu path, it is rendered as a screen-space quad.
///
/// Currently implemented as a stub that stores the most recent stats.
/// Actual pixel drawing is done in `render_into_framebuffer`.
pub struct DebugOverlay {
    pub enabled: bool,
    pub last_stats: FrameStats,
}

impl DebugOverlay {
    pub fn new() -> Self {
        Self {
            enabled: cfg!(debug_assertions),
            last_stats: FrameStats::default(),
        }
    }

    /// Update the stored stats (called once per frame before rendering).
    pub fn update(&mut self, stats: FrameStats) {
        self.last_stats = stats;
    }

    /// Render overlay text into a CPU framebuffer.
    ///
    /// Draws the stats string in white (0xFF_FF_FF_FF) on a semi-transparent
    /// black bar in the top-left corner.
    #[cfg(any(debug_assertions, feature = "debug_overlay"))]
    pub fn render_into_framebuffer(&self, fb: &mut crate::tier0::CpuFramebuffer) {
        if !self.enabled {
            return;
        }
        // Draw a dark banner across the top 12 pixels.
        let banner_height = 12u32;
        for y in 0..banner_height.min(fb.height) {
            for x in 0..fb.width {
                fb.set_pixel(x, y, 0xCC_00_00_00); // semi-transparent black
            }
        }
        // The actual text rendering requires a font rasterizer — stub for now.
        // A minimal 5×7 bitmap font will be added in the next session.
        // [UNRESOLVED: font rasterization for debug overlay]
    }
}

impl Default for DebugOverlay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_stats_fps_derived_correctly() {
        let stats = FrameStats::from_frame_ms(16.666, 2, 100);
        // 1000.0 / 16.666 ≈ 60.0 fps
        assert!((stats.fps - 60.0).abs() < 1.0, "FPS should be ~60, got {}", stats.fps);
        assert_eq!(stats.physics_step_count, 2);
        assert_eq!(stats.entity_count, 100);
    }

    #[test]
    fn display_string_non_empty() {
        let stats = FrameStats::from_frame_ms(8.33, 1, 42);
        let s = stats.to_display_string();
        assert!(s.contains("FPS"));
        assert!(s.contains("42")); // entity count
    }

    #[test]
    fn debug_overlay_update_stores_stats() {
        let mut overlay = DebugOverlay::new();
        let stats = FrameStats::from_frame_ms(20.0, 3, 7);
        overlay.update(stats.clone());
        assert_eq!(overlay.last_stats.physics_step_count, 3);
    }
}
