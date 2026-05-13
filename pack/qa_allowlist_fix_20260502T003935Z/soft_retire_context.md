# Pack — qa_allowlist_fix_20260502T003935Z (Soft Retire)

Session type: Soft retirement
Domain: Process fix (BUG-007); monitoring of parallel reactivations BUG-003, BUG-004, BUG-012
Timestamp: 2026-05-02T00:49:15+05:30
Last commit: b8b8c2c

## Session Work Summary

1. **BUG-007 (process)** — CLOSED
   - Added `## Root Anomaly Allowlist` to `coordinators/quality_gate/PROMPT.md`
   - Lists all 25 permitted root-level entries; `.cursor/` explicitly included
   - Source-of-truth policy: `knowledge/file_structure.md` governs; Tier A + [TIER_A_REVIEW] required to update

2. **BUG-003 (low)** — CLOSED (parallel, user-run session c2_reactivation_20260502T003850Z)
   - `load_components()` replaces `default_components()` in `builder/src/main.rs`
   - Dynamic Cargo.toml reader via toml crate; walk-up heuristic to find workspace root
   - `fem_structural` now correctly surfaces `requires = ["motion_force_simulator"]`

3. **BUG-004 (low)** — CLOSED (parallel, user-run session c2_reactivation_bug004_20260502T004358Z)
   - `format_elapsed(Duration) -> String` added to `component_list.rs`
   - Per-component elapsed label rendered in UI (green/red/grey by status)

4. **BUG-012 (medium)** — CLOSED (parallel, c3_reactivation_bug012_20260502T003829Z)
   - `caps.formats[0]` guarded with `.get(0).copied().unwrap_or(Bgra8UnormSrgb)`

## Project State at Soft Retire

- All 12 bugs in BUG_POOL.md: **CLOSED**
- No open bugs in any severity tier
- `## Pending Claude Review`: empty
- `## Process Violations`: all closed (BUG-010, BUG-011)
- project_manifest.md version: 22
- Last clean commit: b8b8c2c (50 files, 4161 insertions)

## Open Items / Deferred

None. The project is in a fully clean state. All coordinator gates published, all bugs closed.

## New Work If Reactivated

The project is complete. A new session should only be started if:
- A new bug surfaces from a fresh `cargo check --workspace` or test run
- A new feature request arrives from the user (requires new coordinator PROMPT.md via /workflow-coordinator-generator)
- `knowledge/file_structure.md` needs updating (e.g. new root-level directories added by tooling)
