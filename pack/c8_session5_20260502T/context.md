# C8 Session 5 Pack — context.md

Session: c8_session5_20260502T
Model: Claude Sonnet (Tier A)
Status: IN_PROGRESS → COMPLETE (cargo check -p app EXIT:0)

## Work Items Completed

### 1. C8-Viewport: Wire ECS Entity Positions (HIGH) ✅
- Added `Position([f32; 3])` component to `app/src/scene/mod.rs`
- `Scene::get_position(EntityId)` — reads via `world.get_erased()` + downcast
- `Scene::set_position(EntityId, [f32; 3])` — writes via `world.insert_erased()`
- `Scene::entity_positions(limit)` — returns `Vec<(EntityId, [f32; 3])>`
- `ViewportProgram` now carries `entity_positions: Vec<[f32; 3]>`
- `ViewportPrimitive` now carries `entity_positions: Vec<[f32; 3]>`
- `prepare()` uploads real ECS positions into the wgpu point buffer (no more placeholder origin)

### 2. C8-FileFormat: `.fluid` Envelope Save/Load (HIGH) ✅
- `app/src/file/mod.rs` fully rewritten (was stub)
- `FluidEnvelope { format_version, app_version, scene_name, entities }`
- `EntitySnapshot { id, name, position }`
- `save(path, envelope)` — TOML via `toml::to_string_pretty` + `fs::write`
- `load(path)` — `fs::read_to_string` + `toml::from_str`
- `AppMessage::SaveFile(PathBuf)` and `AppMessage::OpenFile(PathBuf)` wired in `app.rs`
- On Open: spawns entities with positions from snapshot, clears history
- On Save: collects `EntitySnapshot` from scene + positions
- 2 unit tests (roundtrip empty scene, roundtrip with entity)

### 3. C8-UI: Outliner Panel (MEDIUM) ✅
- Replaced text labels with `button` rows
- `AppMessage::SelectEntity(EntityId)` dispatched on click
- `selected_entity: Option<EntityId>` field on `FluidApp`
- Selected row: accent background `#6366f1` at 15% alpha
- Hover row: white at 4% alpha
- Wrapped in `scrollable()` for long entity lists

### 4. C8-UI: Properties Panel (MEDIUM) ✅
- No selection: "Select an object to view its properties." hint
- With selection: entity name `text_input` (dispatches `RenameEntity`)
- XYZ position inputs in a row (dispatches `MoveEntity` on each keystroke)
- Property buffers (`prop_name_buf`, `prop_pos_buf: [String; 3]`) populated on `SelectEntity`

## Added Messages
- `AppMessage::SelectEntity(EntityId)`
- `AppMessage::RenameEntity(EntityId, String)`
- `AppMessage::MoveEntity(EntityId, [f32; 3])`
- `AppMessage::SaveFile(PathBuf)`
- `AppMessage::OpenFile(PathBuf)`

## Files Modified
| File | Change |
|------|--------|
| `app/src/scene/mod.rs` | Position component, get_position, set_position, entity_positions, mark_dirty |
| `app/src/viewport/mod.rs` | entity_positions field on ViewportProgram + ViewportPrimitive, real ECS positions in prepare() |
| `app/src/file/mod.rs` | Full rewrite — FluidEnvelope, EntitySnapshot, save(), load(), tests |
| `app/src/app.rs` | SelectEntity/RenameEntity/MoveEntity/SaveFile/OpenFile messages, selected_entity + prop buffers, outliner upgrade, properties panel implementation |

## Validation Gate
cargo check -p app — EXIT:0, 29 warnings (all pre-existing dead_code), 0 errors.

## Next Session (C8 Session 6) Priorities
1. C8-FileFormat: Wire actual file dialog (rfd crate) instead of hardcoded path
2. C8-UI: Menu bar dropdown interactions (File → New/Open/Save)
3. C8-SimBridge: In-process Rayon Tier 0 sim tick integration
4. C8-Import: glTF/OBJ import into Scene via spawn_object + set_position
5. DEC-015: Wrap RenameEntity/MoveEntity/spawn in proper SceneCommand impls for undo
