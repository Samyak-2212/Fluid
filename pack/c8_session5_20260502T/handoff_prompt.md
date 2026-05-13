# C8 Session 6 — Handoff Prompt

You are: **C8, the Fluid GUI Application Coordinator — session 6.**
Model: Claude Sonnet (Tier A)
Domain: `app/`

---

## Mandatory Reading Order

Before any action:

1. `coordinators/app/PROMPT.md` — your specification (read fully)
2. `pack/c8/LATEST.md` — sessions 1–5 + user patch completion state
3. `pack/c8_session5_20260502T/context.md` — session 5 implementation plan
4. `app/DECISIONS.md` — locked architecture decisions
5. `app/INTERFACES.md` — C8↔C9 contracts
6. `bug_pool/BUG_POOL.md` — check before starting
7. `knowledge/project_manifest.md` — overall project state
8. `knowledge/file_structure.md` — file ownership map

---

## Context: Sessions 1–5 + User Patch Complete

### Sessions 1–2 (skeleton + gate)
App skeleton, 5-panel pane_grid, dark theme, debug server, sub-coordinator PROMPT.md files.
`cargo check -p app` EXIT:0. `[C8_INTERFACES_PUBLISHED]` published. C9 unblocked.

### Session 3 (UI wiring)
`iced::application()` wired; 5-panel pane_grid (Viewport3D, Outliner, Properties, Timeline, Console).
Dark theme (#0f0f13 background, #6366f1 accent).

### Session 4 (wgpu viewport)
`iced::widget::shader` + wgpu viewport. `ViewportProgram` (Program trait), `ViewportPrimitive`
(Primitive trait). Orbit/pan/zoom camera (LMB/RMB/scroll). 21×21 grid floor, LineList.
`cargo check -p app` EXIT:0.

### Session 5 (ECS positions + UI)

**C8-Viewport:** `Position([f32; 3])` component added to `app/src/scene/mod.rs`.
`Scene::entity_positions(limit)` collects positions. `ViewportProgram` / `ViewportPrimitive`
carry `entity_positions: Vec<[f32; 3]>`. `prepare()` uploads real ECS coords to wgpu point buffer.

**C8-UI Outliner:** Replaced text labels with `button` rows. `AppMessage::SelectEntity(EntityId)`
dispatched on click. Selected row: accent (`#6366f1`) at 15% alpha. Wrapped in `scrollable`.

**C8-UI Properties:** `selected_entity: Option<EntityId>` on `FluidApp`.
Properties panel shows entity `name` text_input + XYZ position inputs dispatching
`AppMessage::RenameEntity(EntityId, String)` and `AppMessage::MoveEntity(EntityId, [f32;3])`.

**C8-FileFormat:** `FluidEnvelope { format_version, app_version, scene_name, entities }`
+ `EntitySnapshot { id, name, position }`. `AppMessage::SaveFile(PathBuf)` and
`AppMessage::OpenFile(PathBuf)` wired; currently use hardcoded `"scene.fluid"` path.

### User Patch (post-session 5)

`app/src/file/mod.rs` codec switched from TOML (interim) to **`rmp-serde` map-based
MessagePack** per DEC-004 + DEC-011. `rmp_serde::to_vec_named` / `rmp_serde::from_slice`.
Third unit test added: `map_based_encoding_contains_field_names` (DEC-011 compliance gate).
`cargo check -p app` EXIT:0 confirmed after patch.

---

## Critical wgpu API Notes (Session 4 Discovery — still applies)

`iced::widget::shader::wgpu` is **wgpu 0.19.4** — NOT the standalone wgpu 29.0.1 in `app/Cargo.toml`.

| Field | iced wgpu 0.19.4 | standalone wgpu 29 |
|-------|------------------|--------------------|
| `entry_point` | `&str` | `Option<&str>` |
| `VertexState::compilation_options` | **does not exist** | exists |
| `RenderPipelineDescriptor::cache` | **does not exist** | exists |
| `wgpu::util` re-export | **not available** | available |

Always use `iced::widget::shader::wgpu` types in the viewport. Never create a second `wgpu::Instance`.
For buffer uploads use `mapped_at_creation: true` or `queue.write_buffer`.

---

## Critical API Notes (Session 5 Discovery — still applies)

- **`Box<dyn WorldAny>` + `insert()`:** The `World` blanket impl does NOT auto-deref through
  `Box<dyn WorldAny>`. Use `world.insert_erased(entity, TypeId::of::<T>(), Box::new(val))` directly.
  `get_erased()` / `get_erased_mut()` work the same way.
- **`Fn` closures + captured arrays:** `on_input()` closures in iced are `Fn`, not `FnMut`.
  Cannot mutate a captured `[f32; 3]`. Pattern: `let new_pos = [if i==0 {v} else {cur[0]}, ...]`.

---

## Session 6 Work Items (Priority Order)

### 1. C8-FileFormat: File Dialog via `rfd` (HIGH)

Current: `SaveFile(PathBuf)` / `OpenFile(PathBuf)` messages exist but are never sent from the
UI — wired only as stubs. `AppMessage::Save` / `AppMessage::OpenFile` were removed in session 5.

Goal: Wire the menu bar "File → Open" and "File → Save" items to open a native file dialog.

Steps:
- Add `rfd = { version = "0.15", features = ["async-std"] }` to `app/Cargo.toml`
  (check crates.io for latest 0.15.x; `rfd` is cross-platform: Win32 + GTK + Cocoa)
- Add `AppMessage::OpenFileDialog` and `AppMessage::SaveFileDialog` to `AppMessage` enum
- In `update()`, handle these by spawning `rfd::AsyncFileDialog` via `iced::Task::future`:
  ```rust
  AppMessage::OpenFileDialog => {
      return Task::future(async {
          let handle = rfd::AsyncFileDialog::new()
              .add_filter("Fluid Scene", &["fluid"])
              .pick_file()
              .await;
          match handle {
              Some(h) => AppMessage::OpenFile(h.path().to_path_buf()),
              None    => AppMessage::Noop,
          }
      });
  }
  ```
  (same pattern for SaveFileDialog using `.save_file()`)
- Wire `AppMessage::OpenFileDialog` / `AppMessage::SaveFileDialog` from the menu bar
  (see §4 below for menu bar work)
- **No dialog needed for save if a path is already known** — track `current_path: Option<PathBuf>`
  on `FluidApp`; "Save" reuses it, "Save As" always opens the dialog

### 2. C8-UI: Menu Bar Dropdown (MEDIUM)

Current: `view_menu_bar()` renders plain `text()` items — no interaction.

Goal: Make "File" menu functional. Iced 0.13 does not have a built-in dropdown menu widget.
Implement a minimal custom overlay approach:

- Add `menu_open: Option<MenuTarget>` to `FluidApp` where `MenuTarget = File | Edit | ...`
- On click of "File" label, set `menu_open = Some(MenuTarget::File)`; dismiss on any other click
- Render an overlay `container` with the File menu items when `menu_open == Some(File)`:
  - "New Scene" → `AppMessage::NewScene`
  - "Open…"    → `AppMessage::OpenFileDialog`
  - "Save"     → `AppMessage::SaveFileDialog` (or reuse current_path if set)
  - "Save As…" → `AppMessage::SaveFileDialog`
- Use `button` widgets for each item, styled with hover accent

**Constraint:** Do not use any third-party menu crate. Keep it simple — a vertically-positioned
`column` of buttons rendered over the viewport is sufficient for Gate 1.

### 3. C8-SimBridge: In-Process Tier 0 Tick (MEDIUM)

Target: `app/src/sim_bridge/mod.rs` (currently a stub)

Goal: Wire a minimal in-process simulation tick using Rayon (DEC-006, Tier 0).

Steps:
- Define a `SimState { running: bool, tick: u64, dt: f32 }` in `sim_bridge/mod.rs`
- Add `sim_state: SimState` to `FluidApp`
- Wire `AppMessage::SimToggle` to toggle `sim_state.running`
- Wire `AppMessage::SimStep` to advance one tick
- Wire `AppMessage::SimReset` to reset `tick = 0` and clear entity positions to origin
- On `AppMessage::DebugTick` (fires every 100 ms), if `sim_state.running`:
  - Advance `tick += 1`
  - Apply a trivial placeholder orbit: for each entity, rotate position around Y axis by
    `tick as f32 * 0.01` radians — this proves the sim → viewport → GPU pipeline is live
- Update Timeline panel to show `t = tick * dt`

**Constraint:** No physics integration yet (C8-SimBridge's full implementation is a later
session). This is a wiring proof — the goal is a visible live simulation in the viewport.

### 4. DEC-015: SceneCommand Wrappers (LOW)

Current: `RenameEntity` and `MoveEntity` mutate scene state directly in `update()` without
going through `CommandHistory::execute()`. This violates DEC-015.

Wrap the existing direct mutations in `SceneCommand` impls:

- `RenameEntityCmd { entity: EntityId, old_name: String, new_name: String }`
  - `execute`: set `meta.name = new_name`
  - `undo`: set `meta.name = old_name`
- `MoveEntityCmd { entity: EntityId, old_pos: [f32; 3], new_pos: [f32; 3] }`
  - `execute`: `scene.set_position(entity, new_pos)`
  - `undo`: `scene.set_position(entity, old_pos)`

See `app/src/scene/command.rs` for the `SceneCommand` trait and `CommandHistory::execute()` API.
Then update `update()` to call `self.command_history.execute(scene, cmd)` for both messages.

---

## Constraints (Never Violate)

- `iced::widget::shader::wgpu` for all GPU code — never create a second `wgpu::Instance`
- All `unsafe {}` blocks and GPU-touching code: `[NEEDS_REVIEW: claude]`
- Scene mutation via `SceneCommand` + `CommandHistory::execute()` — see `app/src/scene/command.rs`
- `Box<dyn WorldAny>` for ECS access — never downcast to concrete type
- DEC-015: undo/redo must be wirable even if not fully wired yet
- DEC-011: MessagePack MUST use `rmp_serde::to_vec_named` (map-based) — NEVER `to_vec`
- No `unwrap()` in new code — use `?` or explicit `expect("reason")`

---

## Validation Gate

Before retiring this session:

```
cargo check -p app
```

Must exit 0 with **no new errors**. Existing `dead_code` warnings are pre-existing and acceptable.

---

## Pack File Protocol

At 15 tool calls: write `pack/c8_session6_<timestamp>/context.md` and continue.

At retirement:
- Update `pack/c8/LATEST.md`
- Update `knowledge/project_manifest.md` (version bump + commit log entry)
- Update `knowledge/file_structure.md` if >3 files touched

Do NOT publish `[C8_COMPLETE]` until ALL sub-coordinator tasks (C8-UI, C8-Viewport,
C8-FileFormat, C8-Import, C8-SimBridge, C8-Assets) are gate-verified.
