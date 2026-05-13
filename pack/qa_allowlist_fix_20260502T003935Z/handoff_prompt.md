# Handoff Prompt — New Session (Post BUG-007 Close)

Model: Claude Sonnet
Timestamp: 2026-05-02T00:49:15+05:30
Prior session pack: pack/qa_allowlist_fix_20260502T003935Z/

---

## Project State

You are resuming the **Fluid framework** project in a **fully complete and clean state**.

Read these files first, in this exact order:

1. `knowledge/project_manifest.md` — all coordinator gates COMPLETE; version 22
2. `bug_pool/BUG_POOL.md` — all 12 bugs CLOSED; no open items in any tier
3. `knowledge/file_structure.md` — version 9; authoritative root directory list
4. `coordinators/quality_gate/PROMPT.md` — now includes `## Root Anomaly Allowlist`

## What Was Done In Prior Sessions (since C7 retirement)

| Session | Bug | Change |
|---------|-----|--------|
| c1_bugfix_20260502T002703Z | BUG-001 | Split `World` trait into `WorldAny` (object-safe) + `World` (typed blanket); 28 tests pass |
| c3_reactivation_bug012_20260502T003829Z | BUG-012 | Guard `caps.formats[0]` in `rendering/src/surface.rs`; 12 tests pass |
| qa_allowlist_fix_20260502T003935Z | BUG-007 | Added `## Root Anomaly Allowlist` to `coordinators/quality_gate/PROMPT.md` |
| c2_reactivation_20260502T003850Z | BUG-003 | Dynamic `load_components()` replacing hardcoded builder metadata |
| c2_reactivation_bug004_20260502T004358Z | BUG-004 | Per-component elapsed time label in builder UI |

Last clean commit: `b8b8c2c`

## Current Open Bugs

**None.** `bug_pool/BUG_POOL.md` contains zero open entries.

## What To Do In This Session

There is **no pending work** unless the user provides a new task.

If the user asks for new work:
- Check `bug_pool/BUG_POOL.md` first — your bug may already exist
- If adding a new feature/coordinator: use `/workflow-coordinator-generator`
- If a new bug surfaces: file it in `BUG_POOL.md`, assign severity, then fix or hand off
- After touching more than 3 files: update `knowledge/file_structure.md` (currently version 9 — increment to 10 on next write)
- After 15 tool calls: write a soft retire pack and new handoff prompt

## Key File Versions

| File | Version |
|------|---------|
| `knowledge/project_manifest.md` | 22 |
| `knowledge/file_structure.md` | 9 |
| `knowledge/capability_tiers.md` | 2 |
| `knowledge/physics_contract.md` | 1 |
| `knowledge/dependency_graph.md` | 1 |
| `knowledge/model_tier_policy.md` | 1 |
| `knowledge/config_schema.md` | 1 |

## Governance Reminders

- Tier B models: never modify `knowledge/`, `coordinators/*/PROMPT.md`, `ROOT_COORDINATOR.md`
- All commits touching those files require `[TIER_A_REVIEW]` in message
- `knowledge/file_structure.md` is the source of truth for `## Root Anomaly Allowlist` in the QA prompt — update both together
- Hard retirement triggers only on `[CX_INTERFACES_PUBLISHED]` or `[CX_COMPLETE]` gate signals
