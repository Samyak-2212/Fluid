---
description: "Runs the session-recovery skill to reconstruct an interrupted task, verify candidate file integrity, perform minimal evidence-based repair, and resume only after recovery is complete."
---

# Workflow: workflow-session-recovery

## When to Run

Run this workflow when work appears to be resuming after a sudden stop, crash,
tool-budget retirement, interrupted draft, partial write, or uncertain mid-task state.

Do not run for ordinary new tasks unless interruption risk must first be ruled out.

## What This Workflow Does

This workflow activates the `session-recovery` skill, which:

1. confirms this is a real recovery scenario,
2. reconstructs the interrupted task from the strongest available evidence,
3. inspects prior conversation or chat history when available,
4. falls back to pack files, handoff prompts, git/worktree state, and local file
   inspection when chat context is incomplete,
5. builds a narrow list of plausibly interrupted files,
6. verifies whether those files are actually corrupted,
7. repairs only evidence-backed corruption,
8. re-validates repaired files,
9. resumes the original task only after recovery is complete.

## Required Recovery Phases

The workflow must preserve this order:

| Phase | Goal | Must Finish Before Next Phase |
|---|---|---|
| 1. Trigger confirmation | confirm this is sudden-stop recovery | yes |
| 2. Context reconstruction | recover interrupted task from evidence | yes |
| 3. Suspicion scan | identify narrow candidate file set | yes |
| 4. Integrity verification | determine whether candidates are truly corrupted | yes |
| 5. Repair decision | classify each candidate | yes |
| 6. Repair execution | apply only justified repairs | yes |
| 7. Post-repair validation | verify repaired state | yes |
| 8. Task continuation | resume original task | final phase |

The original task must not continue before Phase 7 is complete.

## Evidence Priority

When reconstructing the interrupted task, use this order of reliability:

1. explicit human statement in the new session,
2. prior conversation or chat history,
3. most recent relevant `pack/<id>/context.md`,
4. most recent relevant `pack/<id>/handoff_prompt.md`,
5. local git/worktree state,
6. modified files and current file contents.

If a stronger source conflicts with a weaker one, prefer the stronger source and
note the conflict explicitly.

## Suspicion and Integrity Rules

- Do not assume every modified file is broken.
- Validate suspicious files before classifying them as corrupted.
- Use file-type-appropriate checks where possible.
- Prefer minimal, evidence-based repair over broad rewrites.
- Preserve user and prior-agent work.
- Never discard uncertain edits silently.
- Escalate uncertain repairs to the human before writing.

## Conversation Protocol

Before writing any file:
1. present assumptions and verified constraints,
2. present draft file content,
3. request explicit approval in plain language.

Silence, partial feedback, or implied preference is not approval.

If a suspicious file is not safe to repair automatically, show the proposed repair
or options first and wait for approval.

## Allowed Writes After Approval

This workflow may write only:
- `.agents/skills/session-recovery/SKILL.md`
- `.agents/workflows/workflow-session-recovery.md`
- `pack/<agent_id>_<timestamp>/session_recovery_design_note.md`

No other writes are allowed unless the human explicitly expands scope.

## IDE Portability Rules

The skill and workflow must be usable in agentic coding IDEs without depending on
one IDE's proprietary syntax.

It may refer to canonical repo files and repo-specific operating rules, but should
not require a single IDE’s private command language, hidden metadata format, or
tool-specific markup in order to function.

## Protected Governance Rules

The workflow must not redesign:
- project governance,
- coordinator ownership,
- protected-file rules.

The workflow must not write to:
- `knowledge/`
- `coordinators/`
- `AGENTS.md`
- `ROOT_COORDINATOR.md`
- `.agents/qa/`

## Human Pre-Flight

Before running this workflow, the human should be ready to provide:
- whether this is truly an interrupted-task recovery,
- any known original task statement,
- any known missing context,
- any files they already suspect were interrupted.

The workflow should still proceed with local evidence if those details are incomplete.

## Failure Handling

- If trigger confirmation fails: stop and treat the session as a normal task.
- If context cannot be reconstructed confidently: present the strongest candidate
  reconstruction as `[UNVERIFIED]` and ask for confirmation before any repair.
- If a candidate file is suspicious but intent is uncertain: do not repair without approval.
- If a file is unrecoverable from available evidence: report it clearly and halt
  continuation of the original task.
- If post-repair validation fails: report unresolved state and do not continue.

## Retirement and Handoff

If the running agent reaches 14 tool calls before completing this design task:

1. stop new design work,
2. read `.agents/qa/model_routing_table.md`,
3. write `pack/<agent_id>_<timestamp>/context.md`,
4. write `pack/<agent_id>_<timestamp>/handoff_prompt.md`,
5. present the handoff prompt in conversation as a fenced markdown block,
6. terminate immediately.

The handoff prompt must use the required schema from the skill.
Tier A-approved options for the successor role include Claude Sonnet and GPT-5.4/5.5.
Copy the exact model entry from `.agents/qa/model_routing_table.md`.

## Expected Output

Conversation output order:
1. `Assumptions and Verified Constraints`
2. `Draft: .agents/skills/session-recovery/SKILL.md`
3. `Draft: .agents/workflows/workflow-session-recovery.md`
4. `Draft: pack/<agent_id>_<timestamp>/session_recovery_design_note.md`
5. `Approval Request`

Expected written files after approval:
- `.agents/skills/session-recovery/SKILL.md`
- `.agents/workflows/workflow-session-recovery.md`
- `pack/<agent_id>_<timestamp>/session_recovery_design_note.md`
