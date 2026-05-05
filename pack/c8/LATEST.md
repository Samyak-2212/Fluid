# C8 Pack — LATEST

Session: c8_session6_20260502T
Model: Claude Sonnet (Tier A)
Status: COMPLETE — cargo check -p app EXIT:0, 28 warnings, 0 errors

## Summary

Session 6: Wired native file dialogs (rfd 0.15), functional File menu dropdown,
in-process Tier 0 sim tick with placeholder orbit animation proving the
sim→ECS→viewport→GPU pipeline, and DEC-015 SceneCommand wrappers for all
scene mutations (RenameEntityCmd + MoveEntityCmd through CommandHistory).

## Files Created / Modified

| File | Status | Notes |
|------|--------|-------|
| `app/Cargo.toml` | MODIFIED | Added rfd 0.15 (tokio feature) |
| `app/src/app.rs` | MODIFIED | MenuTarget, OpenFileDialog/SaveFileDialog messages, menu_open + current_path + sim_state fields, full wiring, view_menu_bar dropdown, timeline buttons, apply_sim_orbit() |
| `app/src/scene/command.rs` | MODIFIED | RenameEntityCmd + MoveEntityCmd DEC-015 wrappers |
| `app/src/sim_bridge/mod.rs` | REWRITTEN | SimState (running/tick/dt/orbit), SimBridge stub kept |
| `pack/c8_session6_20260502T/context.md` | NEW | Session 6 pack |

## Key Technical Decisions (Session 6)

- `rfd 0.15` with `tokio` feature — NOT `async-std` (matches iced's tokio runtime; no second runtime)
- `Task::future(async { rfd... })` for non-blocking dialog spawn
- `Task::done(AppMessage::SaveFile(path))` for "Save" shortcut when path already known
- `MenuTarget` enum + `menu_open: Option<MenuTarget>` — no third-party menu crate
- File menu dropdown is a plain `container(column![button...])` rendered below the bar
- DEC-015: RenameEntity/MoveEntity read old value from scene before constructing cmd

## Status at Retirement

- `cargo check -p app`: **EXIT:0**, 28 warnings (all pre-existing dead_code)
- All 4 session 6 work items complete
- [NEEDS_REVIEW: claude] tags preserved on viewport/pipeline.rs and viewport/mod.rs

## Next Session Priorities (C8 Session 7)

1. C8-Import: glTF/OBJ import into Scene via spawn_object + set_position
2. C8-UI: Edit menu dropdown with Undo/Redo items + keyboard bindings
3. C8-Assets: Preset TOML loader (water/air/steel defaults in app/assets/presets/)
4. C8-SimBridge: Autosave on dirty flag at configurable interval
5. C8-UI: Status bar sim state indicator (running/paused/tick rate)

## Soft Retirement

No gate signal published this session. Commit protocol does NOT apply.
Next session: read this file + coordinators/app/PROMPT.md.
