# rendering - Usage Reference

## Architecture Overview

The crate is organized around camera state, rendering backends, and preview output:

```text
rendering
|- camera.rs
|- scene_renderer.rs
|- device.rs
|- surface.rs
|- debug_overlay.rs
|- http_preview.rs
|- pipeline/
\- tier0/
```

`camera.rs` defines view/projection state in terms of `core` units and math. `scene_renderer.rs` provides the renderer trait and a stub implementation. `device.rs` and `surface.rs` wrap `wgpu`. `tier0/` provides a CPU framebuffer and JPEG encoding path for preview output.

## Public API

```rust
pub struct Camera {
    pub position: [Meters; 3],
    pub view: Mat4,
    pub projection: Mat4,
    pub fov_radians: f32,
    pub near: f32,
    pub far: f32,
}
impl Camera {
    pub fn default_perspective(aspect: f32) -> Self;
    pub fn view_projection(&self) -> Mat4;
}

pub struct FrameStats {
    pub frame_ms: f64,
    pub fps: f64,
    pub physics_step_count: u32,
    pub entity_count: usize,
}
impl FrameStats {
    pub fn from_frame_ms(frame_ms: f64, physics_steps: u32, entities: usize) -> Self;
    pub fn to_display_string(&self) -> String;
}

pub struct DebugOverlay {
    pub enabled: bool,
    pub last_stats: FrameStats,
}
impl DebugOverlay {
    pub fn new() -> Self;
    pub fn update(&mut self, stats: FrameStats);
}

pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}
impl GpuContext {
    pub async fn init() -> Result<Self, InitError>;
    pub fn adapter_info(&self) -> String;
}

pub enum InitError {
    NoAdapter,
    DeviceRequest(wgpu::RequestDeviceError),
}

pub type SharedFrame = Arc<Mutex<Vec<u8>>>;
pub fn start_preview_server(port: u16) -> Result<SharedFrame, Box<dyn std::error::Error + Send + Sync>>;
pub fn publish_frame(shared: &SharedFrame, jpeg: Vec<u8>);

pub trait SceneRenderer: Send + Sync {
    fn render<W: fluid_core::ecs::World>(&mut self, world: &W, camera: &Camera);
}

pub struct StubRenderer {
    pub frame_count: u64,
}
impl StubRenderer { pub fn new() -> Self; }

pub struct RenderSurface<'window> {
    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
}
impl<'window> RenderSurface<'window> {
    pub fn new(
        ctx: &GpuContext,
        target: impl raw_window_handle::HasDisplayHandle + raw_window_handle::HasWindowHandle + 'window,
        width: u32,
        height: u32,
    ) -> Result<Self, SurfaceError>;
    pub fn resize(&mut self, ctx: &GpuContext, width: u32, height: u32);
    pub fn format(&self) -> TextureFormat;
}

pub enum SurfaceError {
    Create(wgpu::CreateSurfaceError),
}

pub struct PipelineDescriptor {
    pub label: &'static str,
}

pub type Pixel = u32;
pub struct CpuFramebuffer {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<Pixel>,
}
impl CpuFramebuffer {
    pub fn new(width: u32, height: u32) -> Self;
    pub fn fill(&mut self, argb: Pixel);
    pub fn set_pixel(&mut self, x: u32, y: u32, argb: Pixel);
    pub fn to_jpeg(&self) -> Result<Vec<u8>, image::ImageError>;
}
```

`SoftbufferRenderer` and `pipeline::tier0_pipeline::clear_pass()` are also public behind `tier_0`.

## Configuration

`rendering` consumes `config/rendering.toml`.

| Key | Type | Default | Effect |
|---|---|---|---|
| `preview.preview_http_port` | `u16` | `8080` | Port for the HTTP preview server serving `/frame.jpg`. |
| `frame.width` | `u32` | `1280` | Default headless output width. |
| `frame.height` | `u32` | `720` | Default headless output height. |
| `frame.jpeg_quality` | `u8` | `85` | Target JPEG quality for preview output. [UNVERIFIED: current `CpuFramebuffer::to_jpeg()` does not read this key directly.] |
| `camera.fov_degrees` | `f32` | `45.0` | Default field of view. [UNVERIFIED: `Camera::default_perspective()` currently hardcodes the same value.] |
| `camera.near_clip_meters` | `f32` | `0.1` | Default near clip plane. [UNVERIFIED: not read directly in current source.] |
| `camera.far_clip_meters` | `f32` | `1000.0` | Default far clip plane. [UNVERIFIED: not read directly in current source.] |
| `debug_overlay.force_enabled` | `bool` | `false` | Intended release-build overlay override. [UNVERIFIED: not parsed directly in current source.] |

## Integration with Other Crates

`rendering` consumes `core` for world access, units, and math. The current stub renderer works directly with `core::ecs::ArchetypeWorld`.

```rust
use core::ecs::ArchetypeWorld;
use rendering::{Camera, SceneRenderer, StubRenderer};

fn main() {
    let world = ArchetypeWorld::new();
    let camera = Camera::default_perspective(1.7777778);
    let mut renderer = StubRenderer::new();
    renderer.render(&world, &camera);
}
```

The crate also exposes `start_preview_server()` and `publish_frame()` for debugger or preview workflows that need localhost frame serving.

## Numerical Details

Rendering uses `core::units::Meters` for camera position and relies on `glam` matrices for transforms. The physics contract does not define solver tolerances for rendering, but the crate still inherits workspace rules around SI units and compile-time tiers. Tier 0 uses a pure CPU framebuffer, while Tier 1+ uses `wgpu` without manual backend branching.

## Examples

Render-count smoke test:

```rust
use core::ecs::ArchetypeWorld;
use rendering::{Camera, SceneRenderer, StubRenderer};

fn main() {
    let world = ArchetypeWorld::new();
    let camera = Camera::default_perspective(16.0 / 9.0);
    let mut renderer = StubRenderer::new();
    renderer.render(&world, &camera);
}
```

Preview-server setup:

```rust
use rendering::{publish_frame, start_preview_server};

fn main() {
    let shared = start_preview_server(18080).expect("preview server");
    publish_frame(&shared, Vec::new());
}
```

## Troubleshooting

- If `GpuContext::init()` returns `InitError::NoAdapter`, fall back to the Tier 0 CPU path.
- If `/frame.jpg` returns no content, the preview server may be running before any JPEG bytes have been published.
- If you need text in the debug overlay, note that text rasterization is still unresolved in current source.
