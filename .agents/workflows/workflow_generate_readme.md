---
description: "Activates the generate-readme skill to produce README.md and USAGE.md for all Fluid framework crates. Run as /workflow_generate_readme in a fresh session only."
---

# Workflow: workflow_generate_readme

## When to Run

Run this workflow **only** in a fresh session with no prior tool calls.
Do not run during coordinator tasks, implementation sessions, or debugging sessions.
Invoking this workflow mid-session will produce inaccurate documentation because the
skill pre-execution checklist depends on reading live source files without cached state.

## Trigger

Type exactly:

```
generate-readme
```

as your first and only message in a new session. The system will load the
`generate-readme` skill automatically.

## What This Workflow Does

Executes the `generate-readme` skill, which reads every Fluid crate's source and
knowledge files, then writes accurate `README.md` and `USAGE.md` files for:

- Workspace root
- `core/`
- `components/fluid_simulator/`
- `components/aerodynamic_simulator/`
- `components/motion_force_simulator/`
- `components/thermodynamic_simulator/`
- `components/fem_structural/`
- `rendering/`

## Pre-Flight Checks (human, before starting session)

Before invoking this workflow, confirm:

- [ ] At least one coordinator has published a gate signal (`[CX_INTERFACES_PUBLISHED]`
      or `[CX_COMPLETE]`) in `knowledge/project_manifest.md`. If no gate signals exist,
      documentation will be all `[UNVERIFIED]` — defer until after C1 at minimum.
- [ ] `knowledge/capability_tiers.md` exists and is version ≥ 1.
- [ ] `knowledge/physics_contract.md` exists and is version ≥ 1.
- [ ] `knowledge/config_schema.md` exists and is version ≥ 1.

## Steps

The skill executes autonomously. No human input is required between steps.

| Step | Action | Output |
|------|--------|--------|
| 1 | Read all knowledge/ files listed in skill pre-execution checklist | — |
| 2 | Read `bug_pool/BUG_POOL.md` | — |
| 3 | For each crate in order: read Cargo.toml + src/lib.rs | — |
| 4 | Write `README.md` for crate | File on disk |
| 5 | Write `USAGE.md` for crate | File on disk |
| 6 | Repeat steps 3–5 for all 8 crates | 16 files total |
| 7 | Update `knowledge_b/PROPOSED_doc_status_manifest_section.md` | Status → DRAFT |
| 8 | Output session summary table | Console only |

## Expected Output

- 16 files written (2 per crate × 8 crates), overwriting the Tier B stubs.
- `knowledge_b/PROPOSED_doc_status_manifest_section.md` updated with `DRAFT` status
  and ISO 8601 timestamps.
- A console summary listing each file, byte count, and any `[UNVERIFIED]` items.

## Failure Handling

- If a crate's source files do not yet exist (coordinator not yet run): document what
  is available from `Cargo.toml` only, mark all API sections `[UNVERIFIED — source not yet present]`,
  and continue to the next crate. Do not abort.
- If `knowledge/` files are missing: stop, report which files are missing, and do not
  write any documentation files. Missing knowledge files are a blocker.
- If more than 15 tool calls are consumed before all 16 files are written: write a pack
  file to `pack/generate_readme_<timestamp>/context.md` and hand off per AGENTS.md rules.

## Post-Run Human Review

After the skill completes:

1. Review the console summary for `[UNVERIFIED]` items.
2. Open each `README.md` and `USAGE.md` for spot-check.
3. If content is approved, update `knowledge_b/PROPOSED_doc_status_manifest_section.md`
   status column from `DRAFT` to `APPROVED` for each crate reviewed.
4. A Tier A agent will merge the proposed section into `knowledge/project_manifest.md`
   on the next C7 review cycle.
