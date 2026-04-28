# rendering

Rendering crate for `wgpu` graphics paths, Tier 0 software fallback, and HTTP preview output.

## What It Does

The `rendering` crate is responsible for drawing Fluid scenes and exposing preview output. Its verified source includes a `Camera`, a `SceneRenderer` trait with a `StubRenderer`, `wgpu` device and surface wrappers, a debug overlay stub, and an HTTP preview server that publishes JPEG frames.

This crate sits on top of `core` and uses compile-time tiers to select behavior. Tier 0 uses a CPU framebuffer and optional softbuffer-backed raster path, while higher tiers rely on `wgpu` for GPU backends selected automatically at runtime.

The project manifest still marks the rendering coordinator as blocked and awaiting full completion, so the crate is partially implemented. The public types documented here exist in source, but several files carry `[NEEDS_REVIEW: claude]` markers and some pipeline behavior is still stubbed.

## Capability Tier

| Feature area | Minimum tier | Notes |
|---|---|---|
| CPU framebuffer / Tier 0 preview | 0 | Implemented via `CpuFramebuffer`, `SoftbufferRenderer`, and JPEG export. |
| `wgpu` device and surface setup | 1 | Implemented, with runtime backend selection and review markers in source. |
| Higher-fidelity GPU rendering | 2 | Pipeline scaffolding exists; full render passes are still partial. |
| Coupled Tier 3 compute | 3 | Reserved at architecture level; owned jointly with future component work. |

## Quick Start

```toml
[dependencies]
rendering = { path = "../rendering" }
core = { path = "../core" }
```

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

```bash
cargo build -p rendering
```

## Build Instructions

```bash
cargo build -p rendering
FLUID_TIER=0 cargo build -p rendering
FLUID_TIER=2 cargo build -p rendering
```

Feature flags:

- `tier_0` through `tier_3` are emitted by `rendering/build.rs`.
- `debug_overlay` enables overlay compilation in release builds.

## Known Limitations

- The rendering coordinator has not published `[C3_COMPLETE]`, so the crate remains partially implemented.
- `device.rs`, `surface.rs`, and `pipeline/mod.rs` are marked `[NEEDS_REVIEW: claude]`.
- The debug overlay currently stores stats and paints a banner area, but text rasterization is still `[UNRESOLVED: font rasterization for debug overlay]`.
