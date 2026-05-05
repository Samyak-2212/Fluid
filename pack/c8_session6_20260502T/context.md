# C8 Session 6 Pack — context.md

Session: c8_session6_20260502T
Model: Claude Sonnet (Tier A)
Status: COMPLETE — cargo check -p app EXIT:0, 28 warnings, 0 errors

## Work Items Completed

### 1. C8-FileFormat: rfd native file dialogs (HIGH) ✅
- Added `rfd = { version = "0.15", features = ["tokio"] }` to `app/Cargo.toml`
  (latest 0.15.x = 0.15.4 verified crates.io 2026-05-02; tokio feature matches iced runtime)
- Added `AppMessage::OpenFileDialog` and `AppMessage::SaveFileDialog`
- `OpenFileDialog` spawns `rfd::AsyncFileDialog::pick_file()` via `Task::future`
- `SaveFileDialog`: if `current_path` already set reuses it; else spawns `save_file()` dialog
- `current_path: Option<PathBuf>` on `FluidApp` — set on save/open, cleared on NewScene
- `SaveFile` handler now records `current_path` after successful write

### 2. C8-UI: Menu bar dropdown (MEDIUM) ✅
- Added `MenuTarget` enum: `File | Edit | Simulation | View | Help`
- `menu_open: Option<MenuTarget>` on `FluidApp`
- `AppMessage::MenuOpen(MenuTarget)` / `AppMessage::MenuClose` added
- `view_menu_bar()` replaced text stubs with `menu_button()` helper (accent highlight on open)
- File dropdown: New Scene / Open… / Save / Save As… (no third-party crate)

### 3. C8-SimBridge: In-process Tier 0 tick (MEDIUM) ✅
- `SimState { running, tick, dt }` in `sim_bridge/mod.rs`
  with `maybe_tick()`, `step()`, `reset()`, `time()`, `orbit_position(idx)` helpers
- `sim_state: SimState` on `FluidApp`
- `SimToggle` / `SimStep` / `SimReset` fully wired
- `DebugTick` calls `sim_state.maybe_tick()` → if ticked, calls `apply_sim_orbit()`
- `apply_sim_orbit()` writes placeholder orbit positions to all scene entities
- Timeline panel: ▶/⏸ / ⏭ / ⏹ buttons, `t = X.XXX s`, `Tick N`

### 4. DEC-015: SceneCommand wrappers (LOW) ✅
- `RenameEntityCmd { entity, old_name, new_name }` in `scene/command.rs`
- `MoveEntityCmd { entity, old_pos, new_pos }` in `scene/command.rs`
- `RenameEntity` and `MoveEntity` in `update()` now go through `command_history.execute()`

## Files Modified

| File | Change |
|------|--------|
| `app/Cargo.toml` | Added rfd 0.15 (tokio feature) |
| `app/src/app.rs` | MenuTarget, new messages, new fields, update() wiring, view helpers, apply_sim_orbit(), menu_button() |
| `app/src/scene/command.rs` | RenameEntityCmd + MoveEntityCmd |
| `app/src/sim_bridge/mod.rs` | SimState full implementation |

## Validation Gate

`cargo check -p app` — EXIT:0, 28 warnings (all pre-existing dead_code), 0 errors.

## Next Session Priorities

1. C8-Import: glTF/OBJ import into Scene
2. C8-UI: Edit menu (Undo/Redo), keyboard shortcuts
3. C8-Assets: Preset TOML loader (water/air/steel)
4. C8-SimBridge: Autosave on dirty flag
