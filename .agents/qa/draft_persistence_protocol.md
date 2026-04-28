<!-- QA-AGENT-COMPLETE: issue-3 -->
# Documentation Agent — Draft Persistence Protocol
<!-- Additive protocol. Does not replace the draft-first procedure in   -->
<!-- fluid_readme_skill_prompt.md. Augments the retirement procedure.   -->
<!-- Do not edit without Tier A review.                                 -->

## Problem

Soft retirement at 15 tool calls destroys any in-progress draft that has
not yet been approved and written to disk.

## Required behaviour on soft retirement

When the documentation agent reaches 14 tool calls (one before the limit):

1. If a draft has been presented to the human but not yet approved:
   a. Write the draft verbatim to:
      `knowledge_b/<agent_id>_<timestamp>_pending_draft_<crate_name>.md`
   b. The file must begin with this exact header:
      ```
      <!-- PENDING DRAFT — awaiting human approval -->
      <!-- Crate: <crate_name> -->
      <!-- Agent: <agent_id> -->
      <!-- Session timestamp: <ISO 8601> -->
      <!-- Do not use this file as documentation. It is a draft checkpoint only. -->
      ```
   c. Record the path in the pack file under `[BLOCKED_ON]`:
      `BLOCKED_ON: human approval pending — draft at knowledge_b/<agent_id>_<timestamp>_pending_draft_<crate_name>.md`

2. If no draft is in flight (between crates), `[BLOCKED_ON]` is `nothing`.

## Pickup procedure for the continuing agent

1. Read `pack/<agent_id>_<timestamp>/context.md`.
2. If `[BLOCKED_ON]` references a pending draft file, read that file first.
3. Present it to the human as: "Resuming from previous session.
   The following draft was pending approval:" — then show the draft.
4. Await approval before proceeding. Do not regenerate unless the human
   explicitly requests it.
5. Delete the pending draft file from `knowledge_b/` after the crate is
   written to disk. Commit the deletion.
