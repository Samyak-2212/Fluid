---
description: "Activates the coordinator-generator skill to produce a spec-compliant coordinator PROMPT.md, first-session handoff prompt, and bootstrap context stub for a new Fluid framework coordinator. Run in a fresh session only."
---

# Workflow: workflow-coordinator-generator

## When to Run

Run this workflow **only** in a fresh session with no prior tool calls.
Do not run during coordinator implementation sessions, debugging, or any active
coordinator task. Running mid-session produces inaccurate output because the skill
depends on reading live source files without cached state.

## What This Workflow Does

Executes the `coordinator-generator` skill, which:

1. Reads all required project context files.
2. Asks the human to confirm coordinator type (top-level vs sub-coordinator).
3. Asks a fixed core question set plus applicable type-specific follow-up questions.
4. Validates answers against existing repo constraints.
5. Produces a structured outline and awaits approval (Approval Gate 1).
6. Produces full drafts in conversation and awaits approval (Approval Gate 2).
7. Writes three files to disk only after explicit human approval.
8. Reports written files and surfaces the handoff prompt for immediate use.

**Files written to disk:**
- `coordinators/<name>/PROMPT.md`
- `pack/<name>_<timestamp>/handoff_prompt.md`
- `pack/<name>_<timestamp>/context.md` (bootstrap stub)

**Conversation only — not written to disk:**
- IDE-portable `PROMPT.md` variant

## Pre-Flight Checks (human, before starting session)

- [ ] You know the new coordinator's domain and the files it will own exclusively.
- [ ] You know which existing gate signals (if any) the new coordinator depends on.
- [ ] `knowledge/project_manifest.md` is current and reflects the latest gate signals.
- [ ] `knowledge/dependency_graph.md` is current.
- [ ] `knowledge/model_tier_policy.md` exists and is version ≥ 1.
- [ ] `knowledge/file_structure.md` is current (file ownership must be accurate).
- [ ] `.agents/qa/model_routing_table.md` exists and is current.
- [ ] No existing coordinator owns the files you intend to assign to the new coordinator.

## Steps

| Step | Who | Action | Gate |
|------|-----|--------|------|
| 1 | Agent | Read all pre-read files listed in SKILL.md | — |
| 2 | Agent | Present coordinator type choice (top-level vs sub-coordinator) | Human confirms |
| 3 | Agent | Ask fixed core questions + applicable type-specific follow-ups | — |
| 4 | Human | Answer all questions | — |
| 5 | Agent | Validate answers against repo constraints | Halt on conflict |
| 6 | Agent | Present structured outline | **Approval Gate 1** |
| 7 | Human | Approve outline or request changes | — |
| 8 | Agent | Produce full drafts in conversation | **Approval Gate 2** |
| 9 | Human | Approve drafts or request changes | — |
| 10 | Agent | Write three files to disk | — |
| 11 | Agent | Report written files + surface handoff prompt text | — |

## Human Information to Prepare

Have ready before starting the session:

- Coordinator name / identifier (e.g. "C8 — Audio", "C4-Constraints")
- Exact list of files and directories the coordinator will own exclusively
- Proposed gate signal name
- Prerequisite gate signals from `knowledge/dependency_graph.md`
- Which coordinators this unblocks when its gate signal is published
- Intended model tier (Tier A, Tier B, or split by phase)
- Domain-specific constraints as applicable:
  - Physics/numerical: precision requirements, unsafe code, integrator type
  - Rendering: wgpu pipeline, GPU tier
  - Build/config: TOML schemas, feature flags
  - Debugger/tooling: HTTP endpoints, log format

## Mandatory Approval Gates

**Gate 1 — Outline approval**: The skill presents a structured summary of all
collected information before producing any draft content. The human must explicitly
approve before any draft is produced. Silence, partial feedback, or implied preference
is not approval.

**Gate 2 — Draft approval**: The skill presents all three file drafts plus the
IDE-portable variant in conversation. The human must explicitly approve before any
file is written to disk. Silence is not approval.

## Failure Handling

- **Insufficient answers**: Skill re-asks specific missing items. Does not speculate.
- **Ownership conflict**: Skill halts and reports conflicting coordinator by name.
- **Gate signal conflict**: Skill proposes two alternatives and halts until resolved.
- **Missing pre-read file**: Skill halts and reports missing file. Does not proceed.
- **Human rejects outline or draft**: Skill takes corrections and reproduces the
  rejected artifact. Does not write until approval is explicit.

## Retirement and Handoff

If the agent running this workflow reaches 14 tool calls before writing is complete:

1. Agent stops all work immediately.
2. Agent reads `.agents/qa/model_routing_table.md` to confirm successor model.
3. Agent writes `pack/<agent_id>_<timestamp>/context.md` with current task state.
4. Agent writes `pack/<agent_id>_<timestamp>/handoff_prompt.md`.
5. Agent presents handoff prompt as a fenced markdown block.
6. Agent terminates.

Successor model: Claude Sonnet or GPT-5.4/5.5 (Tier A).

## Expected Output

- 3 files written to disk.
- 1 IDE-portable variant shown in conversation only.
- Output summary: file paths, byte counts, any `[UNRESOLVED]` items.
- Handoff prompt text ready for the human to paste when activating the new coordinator.

## Out of Scope

- Implementing any code for the new coordinator.
- Modifying `knowledge/`, existing coordinator PROMPT.md files, `AGENTS.md`,
  `ROOT_COORDINATOR.md`, `knowledge_b/`, or `.agents/qa/` files.
- Generating coordinator prompts for any existing C1–C7 coordinator.
- Running as a sub-task within another coordinator's active session.
