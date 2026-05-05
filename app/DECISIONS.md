# app/DECISIONS.md
<!-- C8 — Fluid GUI Application: Locked Architecture Decisions -->
<!-- LOCKED: All entries require Tier A (Claude Sonnet) sign-off to change. -->
<!-- Do NOT edit without Tier A review and updated rationale. -->

## DEC-001 through DEC-020 — Session 1 LOCKED

| DEC | Decision | Rationale |
|---|---|---|
| DEC-001 | UI framework: `iced` 0.13+ | Superior aesthetics vs egui; `iced::widget::shader` is stable |
| DEC-002 | 3D viewport: `iced::widget::shader` + wgpu (no Bevy runtime) | `bevy_iced` unmaintained; avoids event loop conflict |
| DEC-003 | Import: `gltf`, `tobj`, `stl_io`, `fbxcel-dom` (no Bevy import dependency) | No Bevy runtime; no second wgpu context. [UNVERIFIED: verify fbxcel-dom maturity] |
| DEC-004 | File format: MessagePack with envelope pattern | Compact binary, schema-versioned |
| DEC-005 | External data: reference-first, per-asset embed policy | Like Blender "Pack Resources" |
| DEC-006 | Sim execution: in-process Tier 0 preview + subprocess Tier 1–3 | Crash isolation for heavy runs |
| DEC-007 | Component plugin interface via `config/component_manifest.toml` | Zero hardcoded component lists |
| DEC-008 | All Tier A (Claude Sonnet) | Risk profile too high for Tier B |
| DEC-009 | Debug server: HTTP on 127.0.0.1:8082, bound to loopback only | Security — not exposed to network |
| DEC-010 | Panel layout: `iced::pane_grid` for tiling (resize + drag). Floating detach = v2 | `pane_grid` confirmed; detach requires custom overlay |
| DEC-011 | MessagePack MUST use map-based struct serialization | Array-based breaks on field reorder |
| DEC-012 | ECS world stored as `Box<dyn WorldAny>`, NOT `Box<dyn World>` | `World` not dyn-compatible (BUG-001 fix) |
| DEC-013 | C8 maintains own widget registry — no AccessKit | Iced 0.13 has zero AccessKit support |
| DEC-014 | STEP (ISO 10303) import deferred to v2 | `ruststep` requires EXPRESS schema work — weeks of effort |
| DEC-015 | Undo/redo via command pattern from day one | `app/src/scene/command.rs` BEFORE any scene mutation code |
| DEC-016 | User preferences: `directories` crate → OS-standard config dir | Win: `%APPDATA%\Fluid\`, Linux: `~/.config/fluid/`, macOS: `~/Library/Application Support/Fluid/`. Separate from `config/`. |
| DEC-017 | Hot-swap settings: `notify` + `arc-swap` via `iced::Subscription` | File watcher MUST use `Subscription::run` + `stream::channel`. NOT bare threads. |
| DEC-018 | Distribution: `.msi` (cargo-wix), `.deb` (cargo-deb), `.dmg` (cargo-bundle) | GitHub Actions matrix. No hardcoded paths in app code. |
| DEC-019 | Tier packaging: Tier 0+1+2 pre-compiled `fluid_sim` binaries in installer. Tier 3 = optional HPC Pack. | App detects hardware, selects tier. User override → restart. |
| DEC-020 | Debug server HTTP library: `tiny_http` (same as C6 debugger) | Minimal overhead; consistent with established C6 pattern |

## Change Protocol

Any change to a LOCKED decision requires:
1. Tier A (Claude Sonnet) sign-off in this file.
2. Update the relevant rows above with new Decision and Rationale.
3. Log the change in `bug_pool/BUG_POOL.md` as a process note or bug (severity: process).
4. Increment `knowledge/file_structure.md` if the change affects file ownership.

## Notes

- DEC-003: [UNVERIFIED: verify fbxcel-dom maturity] — `fbxcel-dom` 0.9.x targets FBX 7.x binary format.
  Verify crate activity on crates.io before implementing C8-Import. If unmaintained, file a bug.
- DEC-012: `WorldAny` is the dyn-compatible trait defined in `core/src/ecs/traits.rs` (BUG-001 resolution).
  Never use `Box<dyn World>` — `World` cannot be made into a trait object.
- DEC-015: `app/src/scene/command.rs` implements the `SceneCommand` trait + `CommandHistory` struct
  before any code that mutates scene state is written. This is a hard ordering constraint.
