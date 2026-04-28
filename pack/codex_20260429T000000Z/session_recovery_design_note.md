# Session Recovery Design Note

## Status

Approved draft written to disk.

## Intent

Add a repo-local `session-recovery` skill and companion workflow that help a future
agent recover safely from a sudden-stop or interrupted session.

## Verified Constraints

- Design only. No code.
- Draft-first in conversation. Await explicit approval before writing.
- Silence, partial feedback, or implied preference do not count as approval.
- No writes to `knowledge/`, `coordinators/`, `AGENTS.md`, `ROOT_COORDINATOR.md`, or `.agents/qa/`.
- Preserve repo-specific rules while keeping the wording usable across agentic coding IDEs.
- The recovery flow must reconstruct context before assuming intent.
- The recovery flow must inspect prior conversation or chat history when available.
- The recovery flow must fall back cleanly to pack files, handoff prompts, git/worktree
  state, and local file inspection when prior chat context is incomplete or unavailable.
- The recovery flow must identify potentially interrupted files, validate whether they are
  actually corrupted, and prefer minimal evidence-based repair.
- The original task must not continue until integrity checks and required repairs are complete.
- Tier A-approved options in repo policy include Claude Sonnet and GPT-5.4/5.5; handoffs
  should copy the exact successor model from `.agents/qa/model_routing_table.md`.

## Design Decisions

1. Recovery is split into eight mandatory phases:
   trigger confirmation, context reconstruction, suspicion scan, integrity
   verification, repair decision, repair execution, post-repair validation,
   and task continuation.

2. Evidence priority is explicit and descending:
   human statement, prior chat, pack context, handoff prompt, git/worktree state,
   then modified files.

3. Suspicion is narrow by default:
   modified files are not presumed broken.

4. Repair is classification-based:
   each candidate file is labeled `clean`, `corrupted and safe to repair automatically`,
   `suspicious but requires human approval`, or `unrecoverable from available evidence`.

5. The workflow is IDE-portable:
   it references canonical repo files and policies but does not depend on one IDE's
   proprietary syntax.

## Open Questions

- [UNRESOLVED] Whether the final skill should mention optional examples of
  file-type-specific integrity checks for additional formats beyond Rust, TOML,
  JSON, Markdown, and YAML.
- [UNRESOLVED] Whether the workflow should include a short human checklist for
  reporting suspected crash symptoms before the agent begins local reconstruction.

## Files Written

- `.agents/skills/session-recovery/SKILL.md`
- `.agents/workflows/workflow-session-recovery.md`
- `pack/<agent_id>_<timestamp>/session_recovery_design_note.md`

## Safe Next Step

Wait for explicit human approval or requested revisions.
Do not write any file until approval is given in plain language.
