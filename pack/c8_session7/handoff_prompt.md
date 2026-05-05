# C8 Session 7 — Handoff Prompt

Model: Claude Sonnet
Coordinator: C8 — GUI Application
Domain: `app/`
Session: c8_session7
Predecessor: c8_session6_20260502T + layout fix (2026-05-04)

---

## Mandatory Reading Order

Before any action:
1. `coordinators/app/PROMPT.md` — your full specification
2. `pack/c8/LATEST.md` — sessions 1–6 + layout fix completion state
3. `pack/c8_session6_20260502T/context.md` — session 6 implementation detail
4. `app/DECISIONS.md` — locked architecture decisions (DEC-001…DEC-020)
5. `app/INTERFACES.md` — C8↔C9 contracts
6. `bug_pool/BUG_POOL.md` — check before starting
7. `knowledge/project_manifest.md` — overall project state (version 30)

---

## State at Handoff

### Confirmed working (cargo build -p app EXIT:0, visually verified)

- 5-panel pane_grid layout: Scene(left) | Viewport(center, 55%) | Properties(right) | Simulation(lower-left) | Timeline(lower-center)
- **Layout fix applied this session**: `build_pane_state()` in `app/src/ui/layout.rs` was starting from Viewport as root; iced's `split()` keeps the original pane first (left), so Viewport ended up at 20%. Fixed by starting from SceneOutliner as root.
- wgpu 3D viewport renders (21×21 floor grid + axis lines visible)
- File menu dropdown: New Scene / Open… / Save / Save As… (rfd 0.15 async dialogs)
- Scene Outliner: clickable entity rows with selection highlight
- Properties panel: live Name + XYZ position text inputs
- Timeline panel: ▶/⏭/⏹ controls, `t=X.XXs`, `#N` tick counter
- Sim: SimToggle/SimStep/SimReset wired; orbit animation on tick (Tier 0 proof)
- DEC-015: RenameEntity + MoveEntity routed through CommandHistory
- Undo/Redo wired (`AppMessage::Undo/Redo` → `command_history.undo/redo`)
- Debug server at 127.0.0.1:8082 (C9 interface — do not break)
- `cargo check -p app`: EXIT:0, 28 warnings (all pre-existing `dead_code`)

### Known remaining dead_code warnings (28 total)

All are intentional scaffolding — not errors:
- `CommandHistory::can_undo/redo`, `peek_undo_label/redo_label`, `undo_labels` → wire to Edit menu this session
- `WidgetRegistry::register/get/clear` → C9 API, already used externally
- `Palette`, `AppTheme`, `BG_SURFACE`, `BORDER`, `WARNING`, `TEXT_MUTED` → future theming
- `ImportFormat`, `ComponentManifest`, `load_manifest` → C8-Import scaffolding
- `SimBridge`, `SimTier::Tier1/2/3`, `binary_path` → Tier 1–3 bridge stub
- `save_prefs`, `despawn_object`, `with_world`, `world/world_mut`, `label` trait fn

---

## Session 7 Priorities (in order)

### 1. C8-UI: Edit menu dropdown + keyboard shortcuts (HIGH)
Wire `CommandHistory::can_undo()` / `can_redo()` / `peek_undo_label()` to an Edit menu dropdown.

**Pattern** (same as File menu in session 6):
```rust
MenuTarget::Edit => { ... }
AppMessage::MenuOpen(MenuTarget::Edit) => { ... }
```

Edit dropdown items:
- `Undo <label>` — greyed if `!command_history.can_undo()`, on_press `AppMessage::Undo`
- `Redo <label>` — greyed if `!command_history.can_redo()`, on_press `AppMessage::Redo`

Keyboard shortcuts: iced 0.13 keyboard subscription via `iced::keyboard::on_key_press`.
- Ctrl+Z → `AppMessage::Undo`
- Ctrl+Y (or Ctrl+Shift+Z) → `AppMessage::Redo`
- Ctrl+N → `AppMessage::NewScene`
- Ctrl+O → `AppMessage::OpenFileDialog`
- Ctrl+S → `AppMessage::SaveFileDialog`

Add the keyboard subscription to `FluidApp::subscription()` alongside the debug tick.

### 2. C8-Import: glTF / OBJ import into Scene (MEDIUM)
Wire the `import/mod.rs` stub (`ImportFormat`, `import()`) to actually load geometry.

Use the `gltf` crate (already in Rust ecosystem, no FFI). For each mesh node:
1. Call `scene.spawn_object(name)` to create an entity
2. Call `scene.set_position(entity, [x, y, z])` with the mesh translation
3. Push a `SpawnEntityCmd` (new command) through `command_history` for undo

Add `AppMessage::ImportFile(PathBuf)` and an Import… item in the File menu.
`rfd` dialog already wired — add `.add_filter("glTF", &["gltf", "glb"])`.

Cargo.toml addition: `gltf = { version = "1", features = ["names", "extras"] }`

### 3. C8-Assets: Preset loader (MEDIUM)
Load material presets from `app/assets/presets/*.toml`. Each preset sets sim parameters (viscosity, density, material) on the selected entity.

Files to create:
- `app/assets/presets/water.toml`
- `app/assets/presets/air.toml`
- `app/assets/presets/steel.toml`

Add `AppMessage::LoadPreset(String)` + Simulation menu items (Simulation dropdown is already in the menu bar but has no items yet).

### 4. C8-SimBridge: Spawn entity from Properties (LOW)
Add a "+" button in the Scene Outliner panel that spawns a new default entity via `SpawnEntityCmd` (from work item 2 above).

```rust
AppMessage::SpawnEntity(name: String)
```

This will reduce the "No objects in scene." empty state.

---

## Architecture Rules (do not violate)

- **DEC-004**: `.fluid` format is binary MessagePack (rmp-serde). `file/mod.rs` already correct.
- **DEC-011**: All serialized structs must be `#[serde(rename_all = "snake_case")]` maps (not tuples).
- **DEC-015**: Every scene mutation through `CommandHistory::execute()`. No direct `scene.set_*/meta_mut()` in `update()`.
- **DEC-001**: iced 0.13.x, wgpu 0.19.4. Do not upgrade.
- **DEC-017**: No bare `std::thread::spawn`. File watchers, imports, and prefs use `Task::future` or `Subscription`.
- **No third-party menu crates**. Dropdowns are plain iced `column` + `button` overlays.
- `dead_code` warnings: do not suppress with `#![allow(dead_code)]` globally. Suppress per-item only when intentional scaffolding is confirmed used-by-spec.

---

## Key File Locations

| File | Purpose |
|------|---------|
| `app/src/app.rs` | Main update/view loop — all messages here |
| `app/src/ui/layout.rs` | Pane grid layout (FIXED this session — read comments before touching) |
| `app/src/scene/command.rs` | SceneCommand trait + CommandHistory + RenameEntityCmd + MoveEntityCmd |
| `app/src/scene/mod.rs` | Scene struct, spawn_object, get/set_position, meta/meta_mut |
| `app/src/file/mod.rs` | FluidEnvelope TOML save/load (DEC-004: must migrate to MessagePack) |
| `app/src/import/mod.rs` | ImportFormat stub — wire glTF here |
| `app/src/sim_bridge/mod.rs` | SimState (Tier 0 orbit tick) |
| `app/src/viewport/` | wgpu pipeline — do NOT modify without [NEEDS_REVIEW: claude] tag |

---

## Gate Condition

`[C8_COMPLETE]` requires all sub-coordinator work items finished AND a runnable binary with no placeholder panels. Current blocker: Import, Assets, and Edit menu dropdowns are still stubs.

Do NOT publish `[C8_COMPLETE]` this session unless all sub-coordinator items in `coordinators/app/PROMPT.md` are satisfied.

After session 7 work: run `cargo check -p app` (must exit 0), update `pack/c8/LATEST.md`, write `pack/c8_session7_.../context.md`.
