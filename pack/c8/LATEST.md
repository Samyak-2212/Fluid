# C8 Pack — LATEST

Session: c8_session8_20260521T
Model: Claude Sonnet (Tier A)
Status: COMPLETE — cargo check -p app EXIT:0, 26 warnings, 0 errors

## Summary

Session 8: Status bar sim state indicator (▶/⏸ + entity count), Delete key →
DespawnEntityCmd (undoable entity deletion), autosave every 60 s on dirty+path,
OBJ import via tobj wired into import_file(), and SimParameters ECS component
with real insert_erased() data path from LoadPreset → World.

## Files Created / Modified

| File | Status | Notes |
|------|--------|-------|
| `app/src/app.rs` | MODIFIED | Priority 1–4: status bar sim state+entity count; DeleteEntity message+handler; AutosaveTick message+60s subscription; OBJ dialog filter; LoadPreset wired to insert_erased(SimParameters) |
| `app/src/import/mod.rs` | MODIFIED | Priority 4: import_obj() via tobj::GPU_LOAD_OPTIONS; Obj arm in import_file() |
| `app/src/sim_bridge/mod.rs` | MODIFIED | Priority 5: SimParameters struct (viscosity, density, material) for ECS storage |
| `knowledge/file_structure.md` | MODIFIED | version 16→17; session 8 files documented |
| `pack/c8/LATEST.md` | MODIFIED | Updated to session 8 |
| `pack/c8_session8_20260521T/context.md` | NEW | This session context |

## Key Technical Decisions (Session 8)

- `keyboard::key::Named::Delete` in the non-modifier branch of `on_key_press` — fires
  `AppMessage::DeleteEntity` which routes through `DespawnEntityCmd` (DEC-015).
  Property buffers are cleared on delete to avoid stale display.
- Autosave uses `iced::time::every(Duration::from_secs(60))` in a third subscription
  (DEC-017 compliant — no bare threads). Only saves if `scene.dirty && current_path.is_some()`.
  Delegates to `Task::done(AppMessage::SaveFile(path))` so the existing save code path is reused.
- `import_obj()` uses `tobj::GPU_LOAD_OPTIONS` (triangulate=true, single_index=true).
  OBJ has no node transforms; all models placed at origin `[0.0, 0.0, 0.0]`.
- `SimParameters` stored via `insert_erased(entity_id, TypeId::of::<SimParameters>(), Box::new(params))`
  — identical pattern to `Position` component storage. Fields are read by future physics
  integrators via `get_erased` + `downcast_ref`.
- Status bar now shows: `Fluid | Tier N | ▶ Running (accent) or ⏸ Paused (muted) | N objects | Frame N`.
- Warning count reduced 28 → 26: `DespawnEntityCmd` is now used (Delete handler),
  `MaterialPreset::viscosity` is now read (SimParameters propagation).

## Status at Retirement

- `cargo check -p app`: **EXIT:0**, 26 warnings (all pre-existing intentional scaffolding)
- Priority 1 (status bar sim state): **DONE**
- Priority 2 (Delete key entity): **DONE**
- Priority 3 (autosave): **DONE**
- Priority 4 (OBJ import via tobj): **DONE**
- Priority 5 (LoadPreset → ECS world): **DONE**

## Known Remaining Dead Code (26 warnings — intentional scaffolding)

- `SimParameters` fields — inserted into ECS but read by future physics bridge
- `SimBridge` struct — subprocess Tier 1–3 scaffold
- `SpawnEntityCmd::entity()` — exposed for future use
- `WidgetRegistry::register/get/clear/unregister`, `WidgetEntry::button/text_input/slider` — C9 API
- `Palette`, `AppTheme`, theme constants — future theming
- `ImportFormat`, `ComponentManifest`, `load_manifest` — stubs
- `SimTier::Tier1/2/3`, `binary_path` — Tier 1–3 bridge stub
- `save_prefs`, `with_world`, `world`, `undo_labels`, `visible`, `entity_count` (ViewportState)

## Next Session Priorities (C8 Session 9)

1. C8-UI: Panel state persistence (save/restore pane sizes to user prefs on disk)
2. C8-UI: `--headless` flag implementation (window suppressed, debug server runs)
3. C8-FileFormat: `cargo test -p app` — expand tests for autosave, file load/save roundtrip
4. C8-Import: STL import via stl_io (follow the OBJ tobj pattern)
5. C8-UI: No hardcoded strings — move all status bar / panel labels to a string table

## Soft Retirement

No gate signal published this session. Commit protocol does NOT apply.
Next session: read this file + coordinators/app/PROMPT.md.
