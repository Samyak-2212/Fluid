# C8 Session 8 — Handoff Prompt
Model: Claude Sonnet
Coordinator: C8 — GUI Application
Domain: `app/`
Session: c8_session8
Predecessor: c8_session7_20260505T
---
## Mandatory Reading Order
Before any action:
1. `coordinators/app/PROMPT.md` — your full specification
2. `pack/c8/LATEST.md` — sessions 1–7 completion state
3. `pack/c8_session7_20260505T/context.md` — session 7 implementation detail
4. `app/DECISIONS.md` — locked architecture decisions (DEC-001…DEC-020)
5. `app/INTERFACES.md` — C8↔C9 contracts
6. `bug_pool/BUG_POOL.md` — check before starting
7. `knowledge/project_manifest.md` — overall project state
---
## State at Handoff
### Confirmed working (cargo check -p app EXIT:0, 28 warnings, 0 errors)
- 5-panel pane_grid layout: Scene(left) | Viewport(center, 55%) | Properties(right) | Simulation(lower-left) | Timeline(lower-center)
- wgpu 3D viewport renders (21×21 floor grid + axis lines visible)
- File menu: New Scene / Open… / Import… / Save / Save As… (rfd 0.15 async dialogs)
- **Edit menu dropdown**: Undo `<label>` (greyed if empty) + Redo `<label>` (greyed if empty)
- **Keyboard shortcuts**: Ctrl+Z → Undo, Ctrl+Y/Shift+Z → Redo, Ctrl+N/O/S wired
- **glTF/GLB import**: `import_file()` → `import_gltf()` → spawns entities via `SpawnEntityCmd`
- **Simulation menu**: lists Water/Air/Steel presets from `app/assets/presets/*.toml`
- **"+" button** in Scene Outliner header → `AppMessage::SpawnEntity` → `SpawnEntityCmd`
- Scene Outliner: clickable entity rows with selection highlight
- Properties panel: live Name + XYZ position text inputs
- Timeline panel: ▶/⏭/⏹ controls, `t=X.XXs`, `#N` tick counter
- Sim: SimToggle/SimStep/SimReset wired; orbit animation on tick (Tier 0 proof)
- DEC-015: RenameEntityCmd + MoveEntityCmd + SpawnEntityCmd + DespawnEntityCmd (scaffolded)
- Undo/Redo wired to CommandHistory
- `PresetDb::load()` — reads `*.toml` from `app/assets/presets/` at startup
- Debug server at 127.0.0.1:8082 (C9 interface — do not break)

### Known remaining dead_code warnings (28 total — intentional scaffolding)
All are pre-existing or intentional new scaffolding. Do not suppress globally.
- `SpawnEntityCmd::entity()` — exposed for future import caller
- `DespawnEntityCmd` / `DespawnEntityCmd::new` — scaffolded, not yet wired to Delete key
- `MaterialPreset::viscosity` — loaded from TOML; wire to World components in C8-SimBridge
- `CommandHistory::can_undo/redo`, `peek_undo_label/redo_label`, `undo_labels` — partially wired; `undo_labels` still unused
- `WidgetRegistry::register/get/clear` → C9 API
- `Palette`, `AppTheme`, theme constants → future theming
- `ImportFormat`, `ComponentManifest`, `load_manifest` → stubs
- `SimBridge`, `SimTier::Tier1/2/3`, `binary_path` → Tier 1–3 bridge stub
- `save_prefs`, `despawn_object`, `with_world`, `world/world_mut`, `label` trait fn

---
## Session 8 Priorities (in order)

### 1. C8-UI: Status bar sim state indicator (HIGH)
Current status bar shows only "Fluid | Tier N | Frame N".
Add:
- Sim state: "▶ Running" (accent) / "⏸ Paused" (muted) based on `sim_state.running`
- Tick rate: derived from `sim_state.tick` and elapsed time
- Entity count: `scene.root_entities().len()`

```rust
// Example status bar row additions:
text(if sim_state.running { "▶ Running" } else { "⏸ Paused" })
    .color(if sim_state.running { accent } else { muted })
```

### 2. C8-UI: Delete entity (HIGH)
Wire `DespawnEntityCmd` (already in command.rs) to:
- A `Delete` key handler in the keyboard subscription (when `selected_entity.is_some()`)
- Add `AppMessage::DeleteEntity` dispatched by the key subscription

```rust
// Keyboard subscription addition:
Key::Named(keyboard::key::Named::Delete) => {
    self.selected_entity.map(|_| AppMessage::DeleteEntity)
}
// Update handler:
AppMessage::DeleteEntity => {
    if let (Some(id), Some(scene)) = (self.selected_entity, self.scene.as_mut()) {
        let name = scene.meta(id).map(|m| m.name.clone()).unwrap_or_default();
        let pos  = scene.get_position(id);
        let cmd  = DespawnEntityCmd::new(id, name, pos);
        let _ = self.command_history.execute(cmd, scene);
        self.selected_entity = None;
    }
}
```

### 3. C8-FileFormat: Autosave (MEDIUM)
Add autosave triggered by a second timer subscription:
- Every 60 seconds (configurable from `UserPrefs` — stub field already exists)
- Only fires if `scene.dirty == true` AND `current_path.is_some()`
- Uses `Task::done(AppMessage::SaveFile(path))` — no dialog

```rust
// In subscription():
let autosave = iced::time::every(Duration::from_secs(60))
    .map(|_| AppMessage::AutosaveTick);
Subscription::batch([tick, keys, autosave])
```

Add `AppMessage::AutosaveTick` handler that calls `SaveFile` if dirty + path known.

### 4. C8-Import: OBJ import via tobj (MEDIUM)
`tobj` is already in `app/Cargo.toml`. Wire it in `import/mod.rs`:

```rust
pub fn import_obj(path: &Path) -> Result<Vec<ImportedMesh>, Box<dyn std::error::Error>> {
    let (models, _materials) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;
    Ok(models.iter().map(|m| ImportedMesh {
        name: m.name.clone(),
        translation: [0.0, 0.0, 0.0], // OBJ has no node transform; origin only
    }).collect())
}
```

Add `ImportFormat::Obj` arm to `import_file()`.
Update the file dialog filter to include `"obj"`.

### 5. C8-SimBridge: Wire LoadPreset to World (LOW)
When `AppMessage::LoadPreset(name)` fires and an entity is selected:
1. Look up `MaterialPreset` from `preset_db`
2. Insert a `SimParameters` component into the ECS world via `world_mut().insert_erased()`
3. Define `SimParameters { viscosity: f64, density: f64, material: String }` in `sim_bridge/mod.rs`

This eliminates the current log-only stub and wires the first real data path from UI → ECS.

---
## Architecture Rules (do not violate)
- **DEC-004**: `.fluid` format is binary MessagePack (rmp-serde). `file/mod.rs` correct.
- **DEC-011**: All serialized structs `#[serde(rename_all = "snake_case")]` maps.
- **DEC-015**: Every scene mutation through `CommandHistory::execute()`.
- **DEC-001**: iced 0.13.x, wgpu 0.19.4. Do not upgrade.
- **DEC-017**: No bare `std::thread::spawn`. File watchers use `Task::future` or `Subscription`.
- **No third-party menu crates**. Dropdowns are plain iced `column` + `button` overlays.
- `dead_code` warnings: suppress per-item only when intentional scaffolding confirmed by spec.

---
## Key Technical Notes from Session 7
- **`dd_style()` free function**: dropdown container style is a free fn, NOT a closure.
  Closures over `Element<'_, _>` cause lifetime error `'1 must outlive '2`. Keep this pattern
  for any new dropdown menus.
- **`Box::leak()` for dynamic menu labels**: only used in the Edit menu where labels change
  per-frame. No steady-state leak (menu closes on click). Do not use elsewhere.
- **`modifiers.command()`**: iced 0.13 cross-platform modifier — Ctrl on Win/Linux, Cmd on macOS.
- **`SpawnEntityCmd` execute order**: `command_history.execute(cmd, scene)` consumes the cmd.
  To read the spawned entity after, use `scene.root_entities().last().copied()` (as done in
  the ImportFile handler).

---
## Key File Locations
| File | Purpose |
|------|---------|
| `app/src/app.rs` | Main update/view loop — all messages here |
| `app/src/ui/layout.rs` | Pane grid layout (FIXED session 6 — read comments before touching) |
| `app/src/scene/command.rs` | SceneCommand trait + CommandHistory + all Cmd impls |
| `app/src/scene/mod.rs` | Scene struct, spawn_object, get/set_position, meta/meta_mut |
| `app/src/file/mod.rs` | FluidEnvelope MessagePack save/load (DEC-004) |
| `app/src/import/mod.rs` | glTF wired; OBJ/STL/FBX stubs |
| `app/src/assets/mod.rs` | PresetDb::load() — reads app/assets/presets/*.toml |
| `app/assets/presets/` | water.toml, air.toml, steel.toml |
| `app/src/sim_bridge/mod.rs` | SimState (Tier 0 orbit tick) |
| `app/src/viewport/` | wgpu pipeline — do NOT modify without [NEEDS_REVIEW: claude] tag |

---
## Gate Condition
`[C8_COMPLETE]` requires all sub-coordinator work items finished AND a runnable binary
with no placeholder panels. Current blockers: Autosave, panel state persistence,
cross-platform build verification, no-hardcoded-strings requirement, tests.
Do NOT publish `[C8_COMPLETE]` this session unless all checklist items in
`coordinators/app/PROMPT.md` are satisfied.

After session 8 work: run `cargo check -p app` (must exit 0), update `pack/c8/LATEST.md`,
write `pack/c8_session8_.../context.md`.
