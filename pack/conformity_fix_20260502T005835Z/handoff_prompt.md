# Handoff Prompt — New Session (Post Conformity Fix)
Model: Claude Sonnet
Timestamp: 2026-05-02T01:12:15+05:30
Prior session pack: pack/conformity_fix_20260502T005835Z/
Prior session commit: 1ac87610436878ab6091e62f4354f4e7596e2494

---

## Project State

You are resuming the **Fluid framework** project in a **fully complete and clean state**.

Read these files first, in this exact order:
1. `knowledge/project_manifest.md` — all coordinator gates COMPLETE; version 22; last SHA 1ac8761
2. `bug_pool/BUG_POOL.md` — all 17 bugs CLOSED; zero open items
3. `knowledge/file_structure.md` — version 10; authoritative root directory list
4. `coordinators/quality_gate/PROMPT.md` — includes `## Root Anomaly Allowlist` (updated this session)

---

## What Was Done In Prior Sessions (full history since C7 retirement)

| Session | Bug(s) | Change |
|---------|--------|--------|
| c1_bugfix_20260502T002703Z | BUG-001 | Split `World` into `WorldAny` (object-safe) + `World` (typed blanket); 28 tests pass |
| c3_reactivation_bug012_20260502T003829Z | BUG-012 | Guard `caps.formats[0]` in `rendering/src/surface.rs`; 12 tests pass |
| qa_allowlist_fix_20260502T003935Z | BUG-007 | Added `## Root Anomaly Allowlist` to `coordinators/quality_gate/PROMPT.md` |
| c2_reactivation_20260502T003850Z | BUG-003 | Dynamic `load_components()` replacing hardcoded builder metadata |
| c2_reactivation_bug004_20260502T004358Z | BUG-004 | Per-component elapsed time label in builder UI |
| **conformity_fix_20260502T005835Z** | **BUG-013–017** | Full design conformity audit + remediation (see below) |

### conformity_fix session detail

- **BUG-013** (process): Added `CLAUDE.md`, `LICENSE`, `.codex/`, `target/` to Root Anomaly Allowlist in `coordinators/quality_gate/PROMPT.md` and `knowledge/file_structure.md` (v10). `[TIER_A_REVIEW]` applied.
- **BUG-014** (low): Documented approved units exception on `NewmarkBetaState` — FEM scalar DOFs use raw `f64` (impractical to wrap matrix math with faer). Approved Tier A.
- **BUG-015** (low): Replaced stale `[NEEDS_REVIEW: claude]` with `[REVIEWED: claude — …]` closure annotations in 19 files. `compute.rs` CUDA/ROCm stubs **retain** the tag.
- **BUG-016** (low): Fixed `knowledge/file_structure.md` self-version row; incremented header 9→10.
- **BUG-017** (low): Guarded `caps.alpha_modes[0]` in `rendering/src/surface.rs` with `.first().copied().unwrap_or(CompositeAlphaMode::Opaque)`.

Committed: `1ac87610436878ab6091e62f4354f4e7596e2494`

---

## Current Open Bugs

**None.** `bug_pool/BUG_POOL.md` contains zero open entries across all 17 bugs (BUG-001 through BUG-017).

---

## What To Do In This Session

There is **no pending work** unless the user provides a new task.

If the user asks for new work:
- Check `bug_pool/BUG_POOL.md` first — your bug may already exist
- If adding a new feature/coordinator: use `/workflow-coordinator-generator`
- If a new bug surfaces: file it in `BUG_POOL.md`, assign severity, then fix or hand off
- After touching more than 3 files: update `knowledge/file_structure.md` (currently version 10 — increment to 11 on next write)
- After 15 tool calls: write a soft retire pack and new handoff prompt

---

## Key File Versions

| File | Version |
|------|---------|
| `knowledge/project_manifest.md` | 22 |
| `knowledge/file_structure.md` | **10** |
| `knowledge/capability_tiers.md` | 2 |
| `knowledge/physics_contract.md` | 1 |
| `knowledge/dependency_graph.md` | 1 |
| `knowledge/model_tier_policy.md` | 1 |
| `knowledge/config_schema.md` | 1 |

---

## Design Conformity Status (post this session)

All checks passing:

| Check | Status |
|---|---|
| Root Anomaly Allowlist complete | ✅ |
| Euler gated `#[cfg(feature = "tier_0")]` | ✅ |
| Newmark/RK4 gated `#[cfg(feature = "tier_1")]` | ✅ |
| ECS `WorldAny`/`World` dyn-safety (BUG-001) | ✅ |
| Surface panic guards: formats + alpha_modes | ✅ |
| NEEDS_REVIEW queue: only `compute.rs` CUDA stubs remain | ✅ |
| `knowledge/file_structure.md` self-consistent | ✅ |
| `NewmarkBetaState` units exception documented | ✅ |

---

## Governance Reminders

- Tier B models: never modify `knowledge/`, `coordinators/*/PROMPT.md`, `ROOT_COORDINATOR.md`
- All commits touching those files require `[TIER_A_REVIEW]` in message
- `knowledge/file_structure.md` is the source of truth for `## Root Anomaly Allowlist` in the QA prompt — update **both** together
- Hard retirement triggers only on `[CX_INTERFACES_PUBLISHED]` or `[CX_COMPLETE]` gate signals
- Before any commit: run `cargo check --workspace` and confirm exit 0
- Before hard retirement: read `.agents/qa/tier_a_commit_protocol.md`
