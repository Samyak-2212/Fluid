//! rendering/src/scene_renderer.rs
//!
//! SceneRenderer trait — consumes the C1 `World` and produces draw calls.
//! Defined per the C3 PROMPT specification.

use crate::camera::Camera;

/// Trait implemented by each tier's concrete renderer.
///
/// `render` is called once per frame after physics is stepped.
/// `world` is the ECS world — the renderer queries it for renderable components.
///
/// # Note on `World` dyn-compatibility
/// `core::ecs::World` is not dyn-compatible because its methods are generic over `C: Component`.
/// `SceneRenderer::render` therefore takes `world` as a concrete type parameter `W: World`
/// rather than `&dyn World`.  This matches the decision documented in C1's gate verification
/// for `core/src/ecs/traits.rs`.
pub trait SceneRenderer: Send + Sync {
    /// Render one frame.
    ///
    /// - `world` — current ECS state.
    /// - `camera` — active camera (view + projection + position in Meters).
    fn render<W: fluid_core::ecs::World>(&mut self, world: &W, camera: &Camera);
}

/// Stub renderer used for testing and as the initial Tier 0 placeholder.
///
/// Does nothing with the world; simply records that `render` was called.
pub struct StubRenderer {
    pub frame_count: u64,
}

impl StubRenderer {
    pub fn new() -> Self {
        Self { frame_count: 0 }
    }
}

impl SceneRenderer for StubRenderer {
    fn render<W: fluid_core::ecs::World>(&mut self, _world: &W, _camera: &Camera) {
        self.frame_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluid_core::ecs::ArchetypeWorld;

    #[test]
    fn stub_renderer_increments_frame_count() {
        let mut renderer = StubRenderer::new();
        let world = ArchetypeWorld::new();
        let camera = Camera::default_perspective(16.0 / 9.0);
        renderer.render(&world, &camera);
        renderer.render(&world, &camera);
        assert_eq!(renderer.frame_count, 2);
    }
}
