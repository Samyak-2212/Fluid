# C8 Pack — LATEST

Session: c8_session7_20260505T
Model: Claude Sonnet (Tier A)
Status: COMPLETE — cargo check -p app EXIT:0, 28 warnings, 0 errors

## Summary

Session 7: Edit menu dropdown with dynamic Undo/Redo labels (greyed when unavailable),
keyboard shortcuts (Ctrl+Z/Y/Shift+Z/N/O/S), glTF/GLB import wired into Scene via
SpawnEntityCmd, material preset TOML loader (water/air/steel), Simulation menu preset
items, "+" spawn button in Scene Outliner header, SpawnEntityCmd/DespawnEntityCmd added
to command.rs.

## Files Created / Modified

| File | Status | Notes |
|------|--------|-------|
| `app/src/app.rs` | MODIFIED | Edit menu (dynamic Undo/Redo), keyboard subscription, ImportFileDialog/ImportFile/LoadPreset/SpawnEntity, Simulation presets menu, "+" outliner button, PresetDb field |
| `app/src/import/mod.rs` | REWRITTEN | glTF/GLB via gltf crate; ImportedMesh struct; OBJ/STL/FBX stubs |
| `app/src/scene/command.rs` | MODIFIED | SpawnEntityCmd + DespawnEntityCmd (DEC-015) |
| `app/src/assets/mod.rs` | NEW | MaterialPreset + PresetDb::load() reads *.toml from app/assets/presets/ |
| `app/assets/presets/water.toml` | NEW | Water preset |
| `app/assets/presets/air.toml` | NEW | Air preset |
| `app/assets/presets/steel.toml` | NEW | Steel preset |
| `app/src/main.rs` | MODIFIED | Added `mod assets;` |
| `knowledge/file_structure.md` | MODIFIED | version 16; new files documented |
| `pack/c8_session7_20260505T/context.md` | NEW | Session 7 pack |

## Key Technical Decisions (Session 7)

- `dd_style()` free function for dropdown container styles (closure caused `'1 must outlive '2` lifetime error).
- `Box::leak()` for dynamic Undo/Redo menu labels (static str requirement; no steady-state leak).
- `Subscription::batch([tick, keys])` — keyboard + debug tick merged.
- `modifiers.command()` — cross-platform (Ctrl on Win/Linux, Cmd on macOS).
- `SpawnEntityCmd` stores `spawned: Option<EntityId>` set at execute-time for exact undo.
- `PresetDb` uses `CARGO_MANIFEST_DIR` in dev; falls back to `<exe_dir>/assets/presets/`.

## Status at Retirement

- `cargo check -p app`: **EXIT:0**, 28 warnings (all pre-existing dead_code)
- Session 7 priorities 1–4: **all DONE**

## Known Remaining Dead Code (28 warnings — intentional scaffolding)

Identical set to session 6 plus new intentional additions:
`SpawnEntityCmd::entity()`, `DespawnEntityCmd` (scaffolded for Delete key), `MaterialPreset::viscosity`

## Next Session Priorities (C8 Session 8)

1. C8-SimBridge: Wire `LoadPreset` to sim parameter propagation (viscosity/density → World components)
2. C8-UI: Status bar sim state indicator (running/paused/tick rate)
3. C8-FileFormat: Autosave on dirty flag at configurable interval
4. C8-UI: Delete entity key — wire DespawnEntityCmd to Delete key / context menu
5. C8-Import: OBJ import via tobj crate

## Soft Retirement

No gate signal published this session. Commit protocol does NOT apply.
Next session: read this file + coordinators/app/PROMPT.md.
