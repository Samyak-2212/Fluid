# app/INTERFACES.md
<!-- C8 — Fluid GUI Application: Sub-coordinator Interface Boundaries -->
<!-- VERSION: 1 (session 1 stub — to be filled as sub-coordinator specs solidify) -->
<!-- Sub-coordinators: READ-ONLY. Do not modify. File bugs if boundaries need revision. -->

## Sub-coordinator Domains

| Sub-ID | Domain | Owned Paths |
|---|---|---|
| C8-UI | Iced pane_grid layout, dark theme, widget registry, TOML theming | `app/src/ui/` |
| C8-Viewport | `iced::widget::shader` + wgpu render pass, camera, raycasting, gizmos, result visualization | `app/src/viewport/` |
| C8-FileFormat | `.fluid` MessagePack envelope (map-based), format_version, migration, path/embed policy | `app/src/file/` |
| C8-Import | glTF/GLB, OBJ, STL, FBX import pipeline; mesh normalization | `app/src/import/` |
| C8-SimBridge | In-process Rayon Tier 0 (`Box<dyn WorldAny>`); subprocess Tier 1–3; catch_unwind; frame cache | `app/src/sim_bridge/` |
| C8-Assets | Preset TOML library, Add-Asset UI flow, material DB, particle emitter presets | `app/src/assets/` |

## Shared Owned by C8 (not sub-coordinator-specific)

| Path | Owner | Notes |
|---|---|---|
| `app/src/main.rs` | C8 | Iced app entry, GPU detect, window init |
| `app/src/scene/` | C8 | Scene graph using `Box<dyn WorldAny>`. command.rs MUST precede any mutation. |
| `app/src/debug_server/` | C8 | `tiny_http` 8082 server, `Arc<RwLock<AppStateSnapshot>>`, widget registry Arc |
| `app/src/prefs/` | C8 | User preferences via `directories` crate + `notify` Subscription + `arc-swap` |
| `app/src/plugin/` | C8 | Component plugin interface reading `config/component_manifest.toml` |
| `app/DECISIONS.md` | C8 | Locked decisions — all subs read, none write |
| `app/INTERFACES.md` | C8 | This file — C8 writes, subs read |
| `app/debug_interface_spec.md` | C8 | Published at [C8_INTERFACES_PUBLISHED] |

## Contract: C8-UI ↔ Debug Server

- C8-UI owns `app/src/ui/widget_registry.rs`.
- The widget registry maps widget ID strings → widget metadata.
- The debug server holds a shared `Arc` clone of the registry.
- Every interactive widget MUST call `.id(iced::widget::Id::new("unique_id"))`.
- Missing IDs: lint warning in debug builds, invisible to C9.
- C8-UI must NOT call into debug server internals — communication is one-directional via Arc.

## Contract: C8-SimBridge ↔ Scene

- Scene graph (`app/src/scene/mod.rs`) holds `Box<dyn WorldAny>` (DEC-012).
- C8-SimBridge receives a shared reference to the scene for in-process Tier 0 execution.
- Tier 1–3 execution is subprocess-only: C8-SimBridge serializes scene → IPC → subprocess.
- C8-SimBridge MUST use `catch_unwind` around in-process simulation steps.
- Frame results are cached in `app/src/sim_bridge/frame_cache.rs` (planned).

## Contract: C8-Viewport ↔ Scene

- C8-Viewport reads scene state (read-only) each frame for rendering.
- Raycasting results (selection) are communicated back via scene commands (DEC-015).
- C8-Viewport MUST NOT mutate scene state directly — all mutations go through CommandHistory.

## Contract: C8-FileFormat ↔ Scene

- Serialization: C8-FileFormat serializes the scene to `.fluid` MessagePack format.
- DEC-011: map-based serialization only (NOT array-based).
- format_version field in envelope — migration adapters handle version deltas.
- External assets: reference by path by default; embed on explicit user action (DEC-005).

## Contract: C8 ↔ C9 (Agent Debugger)

- C9 connects to `127.0.0.1:8082` (configurable in `config/app.toml`, default port per DEC-009).
- C9 uses GET /health first to verify protocol_version.
- C9 reads widget state via GET /tree — only `.id()`-tagged widgets appear.
- C9 sends control actions via POST /control with a 5-second timeout.
- C9 MUST handle the headless case: `/screenshot` returns error JSON, `/tree` returns empty.
- The full contract is published in `app/debug_interface_spec.md` at [C8_INTERFACES_PUBLISHED].

## Interface Stability

- Interfaces in this file are PLANNED (session 1 stub) until marked **[STABLE]**.
- Sub-coordinators MUST file a bug before changing any cross-sub contract.
- C8 is the arbiter — all cross-sub disputes are resolved by C8 or escalated to root.
