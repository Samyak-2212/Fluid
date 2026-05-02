# C8 Pack — LATEST

Session: c8_session4_20260502T
Model: Claude Sonnet (Tier A)
Status: COMPLETE — cargo check -p app EXIT:0

## Summary

Implemented `iced::widget::shader` wgpu viewport integration for C8-Viewport.

## Files Created / Modified

| File | Status | Notes |
|------|--------|-------|
| `app/src/viewport/camera.rs` | REWRITTEN | Orbit camera: spherical coords, view_proj(), orbit/pan/zoom |
| `app/src/viewport/pipeline.rs` | NEW | [NEEDS_REVIEW: claude] wgpu LineList grid + PointList entity pipelines |
| `app/src/viewport/mod.rs` | REWRITTEN | [NEEDS_REVIEW: claude] ViewportProgram (Program), ViewportPrimitive (Primitive), DragState |
| `app/src/app.rs` | PATCHED | view_viewport_panel() → iced::widget::shader(program) |
| `app/Cargo.toml` | PATCHED | added bytemuck = {version="1", features=["derive"]}; iced advanced feature |
| `knowledge/project_manifest.md` | UPDATED | version 27; session 4 commit log; C8 status updated |
| `knowledge/file_structure.md` | UPDATED | version 14; viewport files documented |
| `pack/c8_session4_20260502T/context.md` | NEW | mid-session plan stub |

## Key Technical Decisions

- `iced::widget::shader::wgpu` is wgpu **0.19.4** (not 29.0.1) — the iced re-export uses an older API
  - `entry_point: &str` (not `Option<&str>`)
  - No `compilation_options` field on VertexState/FragmentState
  - No `cache` field on RenderPipelineDescriptor
  - `Operations { load, store: StoreOp }` — StoreOp is an enum
  - `wgpu::util` is NOT re-exported — use `mapped_at_creation: true` for buffer upload
- `iced::advanced` feature must be enabled to access `Shell` for `Program::update()`
- `Status` is from `iced::widget::canvas::event::Status` (not `iced::event::Status`)
- Camera state lives in `ViewportInteractState` (Program's associated State) — managed by iced runtime
- Grid: 21×21 lines on XZ plane, LineList topology, axis lines appended (red X, blue Z)
- Placeholder entity markers: PointList at origin, one per entity (session 5 will use ECS positions)

## API Confirmed

```rust
Program<Message> {
    type State: Default + 'static;       // ViewportInteractState
    type Primitive: Primitive + 'static; // ViewportPrimitive
    fn draw(&self, state, cursor, bounds) -> Primitive;
    fn update(&self, state, event, bounds, cursor, shell) -> (Status, Option<Message>);
    fn mouse_interaction(&self, state, bounds, cursor) -> Interaction;
}

Primitive {
    fn prepare(&self, device, queue, format, storage, bounds, viewport);
    fn render(&self, encoder, storage, target: &TextureView, clip_bounds: &Rectangle<u32>);
}
```

## Status at Retirement

- `cargo check -p app`: **EXIT:0**, warnings only (35 pre-existing dead_code, none from viewport)
- No open bugs affecting C8-Viewport
- [NEEDS_REVIEW: claude] tags applied to both pipeline.rs and mod.rs as required

## Next Session Priorities (C8-FileFormat or C8-UI polish)

1. C8-FileFormat: `.fluid` envelope save/load (app/src/file/)
2. C8-UI: panel resize handles, outliner tree, properties grid
3. Session 5 viewport: wire ECS entity positions from `Scene::world()` into point buffer

## Soft Retirement

No gate signal published this session. Commit protocol does NOT apply.
Next session: read this file + coordinators/app/PROMPT.md + pack/c8/LATEST.md.
