# C8 Session 5 — Handoff Prompt

You are: **C8, the Fluid GUI Application Coordinator — session 5.**
Model: Claude Sonnet (Tier A)
Domain: `app/`

---

## Mandatory Reading Order

Before any action:

1. `coordinators/app/PROMPT.md` — your specification (read fully)
2. `pack/c8/LATEST.md` — session 4 completion state
3. `pack/c8_session4_20260502T/context.md` — session 4 implementation plan
4. `app/DECISIONS.md` — locked architecture decisions
5. `app/INTERFACES.md` — C8↔C9 contracts
6. `bug_pool/BUG_POOL.md` — check before starting
7. `knowledge/project_manifest.md` — overall project state
8. `knowledge/file_structure.md` — file ownership map

---

## Context: Sessions 1–4 Complete

### Session 1 (skeleton)
App skeleton: `main.rs`, `app.rs` (FluidApp, AppMessage, 5-panel pane_grid), all module stubs, dark theme, debug server, sub-coordinator PROMPT.md files, `config/app.toml`.

### Session 2 (gate)
Fixed `fbxcel-dom` version; published `[C8_INTERFACES_PUBLISHED]`; C9 unblocked.

### Session 3 (UI)
`iced::application()` wired; 5-panel pane_grid (Viewport3D, Outliner, Properties, Timeline, Console); dark theme (#0f0f13 background, #6366f1 accent). `cargo check -p app` EXIT:0.

### Session 4 (viewport)
**Fully implemented `iced::widget::shader` wgpu viewport.** `cargo check -p app` EXIT:0, warnings only.

Files written:
- `app/src/viewport/camera.rs` — orbit camera (LMB=orbit, RMB=pan, scroll=zoom)
- `app/src/viewport/pipeline.rs` [NEEDS_REVIEW: claude] — wgpu LineList grid + PointList entity pipelines, mapped_at_creation upload
- `app/src/viewport/mod.rs` [NEEDS_REVIEW: claude] — ViewportProgram (Program trait), ViewportPrimitive (Primitive trait), DragState, ViewportInteractState
- `app/src/app.rs` — view_viewport_panel() uses iced::widget::shader
- `app/Cargo.toml` — bytemuck derive, iced `advanced` feature

---

## Critical wgpu API Notes (Session 4 Discovery)

`iced::widget::shader::wgpu` is **wgpu 0.19.4** — NOT the standalone wgpu 29.0.1 in app/Cargo.toml.

| Field | iced wgpu 0.19.4 | standalone wgpu 29 |
|-------|------------------|--------------------|
| `entry_point` | `&str` | `Option<&str>` |
| `VertexState::compilation_options` | **does not exist** | exists |
| `RenderPipelineDescriptor::cache` | **does not exist** | exists |
| `wgpu::util` re-export | **not available** | available |

Always use `iced::widget::shader::wgpu` types in the viewport. Do NOT mix types with the standalone `wgpu` crate.

For buffer uploads use `mapped_at_creation: true` or `queue.write_buffer` — never `wgpu::util::DeviceExt`.

---

## Session 5 Work Items (Priority Order)

### 1. C8-Viewport: Wire ECS Entity Positions (HIGH)

Current: entity marker is a single point at origin (placeholder from session 4).
Goal: read entity world positions from `Scene::world()` → pass into `prepare()` via `ViewportPrimitive`.

Steps:
- `Scene::world()` returns `&dyn WorldAny` (from `core/src/ecs/traits.rs`)
- `WorldAny::entity_ids()` returns a list of live entity IDs
- For now, use default position (0,0,0) per entity until a `Position` component is defined
- Add a `Position` component to `app/src/scene/mod.rs` (simple `[f32; 3]`)
- Query it via `WorldAny::get_component_raw()` if available, else keep placeholder
- Cap at `MAX_SPHERE_POINTS = 256` (already in pipeline.rs)

### 2. C8-FileFormat: `.fluid` Envelope Save/Load (HIGH)

Target files: `app/src/file/mod.rs` (currently stub)

Spec: `app/DECISIONS.md` DEC-003 (TOML envelope, glTF mesh data)

Implement:
- `FluidEnvelope` struct: `version: u32`, `scene_name: String`, `entities: Vec<EntitySnapshot>`
- `EntitySnapshot`: `id: u64`, `name: String`, `position: [f32; 3]`
- `save(path, envelope) -> Result<(), Box<dyn Error>>` — serialize to TOML
- `load(path) -> Result<FluidEnvelope, Box<dyn Error>>` — deserialize from TOML
- Use `serde` + `toml` crate (add to `app/Cargo.toml` if not present)
- Wire `AppMessage::SaveFile(PathBuf)` and `AppMessage::OpenFile(PathBuf)` in `app.rs`
- No actual filesystem dialog yet — use hardcoded `"scene.fluid"` path for now

### 3. C8-UI: Outliner Panel (MEDIUM)

Target: `view_outliner_panel()` in `app/src/app.rs` (currently shows placeholder text)

Implement a scrollable tree of entities:
- Query `scene.entities()` → list of `(EntityId, ObjectMeta)`
- Render as a `column![]` of `button![]` rows, each showing entity name
- Clicking selects entity → `AppMessage::SelectEntity(EntityId)`
- Add `selected_entity: Option<EntityId>` to `FluidApp`
- Selected row gets accent background (`#6366f1` at 15% alpha)

### 4. C8-UI: Properties Panel (MEDIUM)

Target: `view_properties_panel()` in `app/src/app.rs`

Show properties of `selected_entity`:
- If None: display "No selection" hint
- If Some: show entity name (editable text input), position fields (3× f32 inputs)
- Wire `AppMessage::RenameEntity(EntityId, String)` and `AppMessage::MoveEntity(EntityId, [f32;3])`

---

## Constraints (Never Violate)

- `iced::widget::shader::wgpu` for all GPU code — never create a second `wgpu::Instance`
- All `unsafe {}` blocks and GPU-touching code: `[NEEDS_REVIEW: claude]`
- Scene mutation via `SceneCommand` + `CommandHistory::execute()` — see `app/src/scene/command.rs`
- `Box<dyn WorldAny>` for ECS access — never downcast to concrete type
- DEC-015: undo/redo must be wirable even if not fully wired yet
- No `unwrap()` in new code — use `?` or explicit `expect("reason")`

---

## Validation Gate

Before retiring this session:

```
cargo check -p app
```

Must exit 0 with **no new errors**. Existing dead_code warnings are pre-existing and acceptable.

---

## Pack File Protocol

At 15 tool calls: write `pack/c8_session5_<timestamp>/context.md` and continue.
At retirement: update `pack/c8/LATEST.md`, update `knowledge/project_manifest.md` (version bump + commit log), update `knowledge/file_structure.md` if >3 files touched.

Do NOT publish `[C8_COMPLETE]` until ALL sub-coordinator tasks (C8-UI, C8-Viewport, C8-FileFormat, C8-Import, C8-SimBridge, C8-Assets) are gate-verified.
