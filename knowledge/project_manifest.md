<!-- version: 4 -->
# Project Manifest

## Project: Fluid
Language: Rust (edition 2021+)
Root session: root_coordinator_20260427T032847Z
Status last updated: 2026-04-27T03:28:47+05:30

---

## Coordinator Status

| Coordinator | Domain | Status | Gate Signal | Notes |
|-------------|--------|--------|-------------|-------|
| C1 — Core Systems | `core/` | COMPLETE | `[C1_INTERFACES_PUBLISHED]` ✅ `[C1_COMPLETE]` ✅ | All impl done; 26 tests pass |
| C2 — Build System | `builder/` | COMPLETE | `[C2_COMPLETE]` ✅ | cargo build -p builder: 0 errors, 0 warnings |
| C3 — Rendering | `rendering/` | BLOCKED | `[C3_COMPLETE]` pending | Waiting on C1 |
| C4 — Physics Core | `physics_core/` | BLOCKED | `[C4_INTERFACES_PUBLISHED]` pending | Waiting on C1 |
| C5 — Sim Components | `components/` | BLOCKED | `[C5_COMPLETE]` pending | Waiting on C4 |
| C6 — Debugger | `debugger/` | BLOCKED | `[C6_COMPLETE]` pending | Waiting on C1+C2 |
| C7 — Quality Gate | (cross-cutting) | NOT_STARTED | `[C7_COMPLETE]` pending | Can begin setup with C1+C2 |
| Root | (this file) | IN_PROGRESS | `[ROOT_COMPLETE]` pending | Writing final manifest |

---

## Completion Gate Signals

Signals written here by coordinators upon reaching their gates.
Writing any `[CX_INTERFACES_PUBLISHED]` or `[CX_COMPLETE]` signal is a hard retirement trigger.

<!-- Signals are appended below as coordinators complete their gates -->

[C1_INTERFACES_PUBLISHED]
Published by: C1 (session: c1_core_20260428T022400Z)
Timestamp: 2026-04-28T02:24:00+05:30
Gate files verified:
- core/src/units.rs — 13 SI newtype wrappers, all required types present, 8 unit tests pass
- core/src/ecs/traits.rs — EntityId, Component, World, System<W> defined; System uses W: World type param (not dyn World — World is not dyn-compatible due to generic methods; documented in trait)
- core/src/event_bus.rs — Event blanket impl + EventBus trait defined
cargo test -p core: 13 passed, 0 failed, EXIT:0

C3, C4, C6 may now begin.

---

[C1_COMPLETE]
Published by: C1 (session: c1_continuation_20260428T080200Z)
Timestamp: 2026-04-28T08:02:00+05:30
All C1 post-gate work verified:
- config/core.toml: timestep_seconds, max_ticks_per_frame, rayon_num_threads
- core/src/math/mod.rs: glam 0.32.1 verified, [UNVERIFIED] removed
- core/src/threading/traits.rs: rayon 1.12.0 verified, [UNVERIFIED] removed
- core/src/ecs/world.rs: ArchetypeWorld concrete impl, 8 tests pass [NEEDS_REVIEW: claude]
- core/src/event_bus_impl.rs: LocalEventBus concrete impl, 5 tests pass [NEEDS_REVIEW: claude]
cargo test -p core: 26 passed, 0 failed, EXIT:0
BUG-005 filed for C2 (workspace.edition warning)

C1 domain closed. File new core/ bugs to BUG_POOL.md, assign to C7 for triage.

---

[C2_COMPLETE]
Published by: C2 (session: c2_build_system_20260429T173700Z)
Timestamp: 2026-04-29T17:41:00+05:30
Gate files verified:
- builder/src/main.rs — FluidBuilderApp, eframe 0.34.1 fn ui + fn logic
- builder/src/subprocess.rs — BuildProcess: spawn, poll_output, kill, is_running, exit_status; child.kill() only
- builder/src/config.rs — FlagEntry/BuilderConfig/FlagState; typed TOML, no panic on missing keys
- builder/src/state.rs — ComponentStatus, BuildSessionState, BuildState
- builder/src/ui/ — flag_panel.rs, output_panel.rs, component_list.rs
- config/builder_flags.toml — 7 initial flags populated
cargo build -p builder: 0 errors, 0 warnings, Exit: 0
BUG-002 CLOSED. BUG-005 CLOSED. BUG-008 filed and closed (rendering wgpu gl→gles).

C6 (Debugger) and C7 (Quality Gate) may now begin.

---

## Unresolved Items

| Tag | Location | Description | Owner |
|-----|----------|-------------|-------|
| `[UNRESOLVED]` | `knowledge/capability_tiers.md` | oneAPI (Intel) Tier 3 compute support — feasibility not assessed | C5 |

---

## In-Progress Sessions

<!-- Format: [IN_PROGRESS: <agent_id> at <timestamp> on <task>] -->
<!-- Remove when session retires. C7 audits for stale entries. -->
<!-- C1 session retired on gate signal write — no in-progress entry remains -->

---

## Retired Sessions

<!-- Format: [RETIRED: <agent_id> at <timestamp>] -->
<!-- Permanent record. Never delete. -->

[RETIRED: c1_core_20260428T022400Z at 2026-04-28T02:24:00+05:30]

---

## Reactivated Sessions

<!-- Format: [REACTIVATED: <agent_id> at <timestamp> for BUG-<id>] -->

---

## Incremental Commit Log

<!-- Format: <timestamp> <agent_id> <file count> <line count> <description> -->
<!-- Flag any commit touching >400 lines across >5 files without coordinator sign-off -->

---

## Notes

- `config/builder_flags.toml` must be created by C2 before any component coordinator adds flags.
- C5 oneAPI `[UNRESOLVED]` must be resolved before `[C5_COMPLETE]` is published.
- C7 must confirm all `[RETIRED]` entries have a corresponding `handoff_prompt.md` in `pack/`.
- knowledge/ files must not be written without incrementing the `<!-- version: N -->` counter.

[ROOT_COMPLETE]
