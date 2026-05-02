# C8 Pack — Session 4 (mid-session checkpoint)

Session: c8_session4_20260502T
Model: Claude Sonnet (Tier A)
Status: IN PROGRESS — reading complete, implementation pending

## Work planned this session (C8-Viewport)

### Files to write / rewrite

| File | Change |
|------|--------|
| `app/src/viewport/mod.rs` | REWRITE — ViewportState + ViewportProgram + ViewportProgram::State |
| `app/src/viewport/pipeline.rs` | NEW — ViewportPipelineState (GPU resources in Storage) |
| `app/src/viewport/camera.rs` | ENHANCE — add view_proj() matrix builder |
| `app/src/app.rs` | PATCH — view_viewport_panel() uses iced::widget::shader |
| `app/Cargo.toml` | PATCH — add bytemuck dep |

### API confirmed from docs.rs

```
Program<Message>:
  type State: Default + 'static        // per-widget interactive state (camera orbit)
  type Primitive: Primitive + 'static  // returned from draw()
  fn draw(&self, state, cursor, bounds) -> Primitive  // required
  fn update(&self, state, event, bounds, cursor, shell) -> (Status, Option<Message>)  // optional

Primitive (from iced_wgpu::primitive):
  fn prepare(&self, device, queue, format, storage, bounds, viewport)
  fn render(&self, encoder, storage, target: &TextureView, clip_bounds: &Rectangle<u32>)
```

### Design decisions

- `ViewportProgram` is a lightweight struct constructed inline in view_viewport_panel()
  Fields: entity_count: u64 (scene snapshot for this frame)
- `ViewportProgram::State` holds: camera: Camera, drag_state: DragState (orbit/pan/idle)
- `ViewportPrimitive` holds: camera_data (MVP matrix), entity_count, scene_info
- `ViewportPipelineState` (stored in Storage) holds: pipelines, buffers, bind groups
- NO second wgpu Device — device provided via prepare()
- Camera orbit: LMB drag; Pan: RMB drag; Zoom: scroll wheel
- Grid: XZ plane -10..+10, line list topology, 42 lines (84 vertices)
- Placeholder spheres: point list at origin per entity (entity_count > 0)
- Clear color: #0f0f13 = (0.0588, 0.0588, 0.0745, 1.0)
- All unsafe{} tagged [NEEDS_REVIEW: claude]
- WGSL shaders inline (include_str! of embedded constants)
- bytemuck for uniform buffer writes (Pod/Zeroable)

## Bug check
No open bugs in BUG_POOL.md affecting C8-Viewport.

## Next: write implementation files
