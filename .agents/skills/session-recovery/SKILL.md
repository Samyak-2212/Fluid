---
name: session-recovery
description: >
  Recovers an interrupted agent session by reconstructing the original task
  from evidence, validating file integrity, repairing only evidence-backed
  corruption, and resuming work only after recovery is complete. Use for
  sudden-stop or interrupted-session recovery in agentic coding IDEs.
---

# session-recovery Skill

## Activation

Activate this skill only when the current session appears to be continuing work
that was interrupted unexpectedly or ended mid-task.

Do not activate for:
- normal greenfield requests,
- routine follow-up work with no interruption risk,
- broad cleanup requests that are not tied to a prior interrupted task,
- requests whose main goal is redesigning repo governance.

If the scenario is ambiguous, begin with Trigger Confirmation and do not assume.

## Purpose

This skill restores a trustworthy working state after an interrupted session.

It does this in four strict stages:
1. confirm that recovery is actually needed,
2. reconstruct the interrupted task from evidence,
3. inspect potentially interrupted files for real corruption,
4. repair only what is justified, validate, and only then resume the original task.

## Scope

In scope:
- sudden-stop recovery for interrupted coding or documentation sessions,
- reconstructing task intent from human statement, chat history, packs, handoffs,
  git/worktree state, and local file contents,
- narrow integrity verification on files plausibly interrupted mid-write,
- minimal evidence-based repair,
- draft-first review of any proposed repair or recovery artifacts before writing.

Out of scope:
- inventing a new task when the interrupted task cannot be reconstructed,
- broad refactors disguised as recovery,
- silently reverting uncertain edits,
- rewriting protected governance files,
- continuing implementation before integrity verification is complete.

## Non-Scope Protected Files

This skill must not write to:
- `knowledge/`
- `coordinators/`
- `AGENTS.md`
- `ROOT_COORDINATOR.md`
- `.agents/qa/`

If recovery analysis suggests one of those files may be damaged, report it in
conversation and require explicit human direction. Do not modify it.

## Required Recovery Logic

The skill must execute these phases in order.

### 1. Trigger Confirmation

Confirm that this is a sudden-stop recovery scenario rather than a normal new task.

Strong indicators include:
- the human explicitly says work was interrupted, crashed, stopped, or needs recovery,
- there is unfinished draft or pack evidence from a prior session,
- the worktree shows partial edits that align with an interrupted task,
- files show incomplete write patterns consistent with a cut-off edit.

If recovery is not confirmed, stop and fall back to normal task handling.
Do not run the rest of this skill spec.

### 2. Context Reconstruction

Recover the original task from the strongest available evidence in this order:

1. explicit human statement in the new session,
2. prior conversation or chat history available to the current agent or IDE,
3. most recent relevant `pack/<id>/context.md`,
4. most recent relevant `pack/<id>/handoff_prompt.md`,
5. local git/worktree state,
6. currently modified files and their contents.

Rules:
- Inspect prior conversation or chat history when it is available.
- If prior chat context is incomplete or unavailable, fall back cleanly to pack
  files, handoff prompts, git status, and local file inspection.
- State which sources were available and which were not.
- Prefer direct evidence over inference.
- Mark inferred conclusions as `[UNVERIFIED]`.

Before continuing, produce a short reconstruction summary:
- interrupted task,
- strongest evidence used,
- missing context,
- current confidence level.

### 3. Suspicion Scan

Build a narrow candidate list of files that may have been interrupted mid-write.

Candidate signals may include:
- files modified very recently relative to the interrupted session evidence,
- files with obvious truncation clues,
- files named in pack or handoff context,
- files with partial markers such as unfinished headings, half-written blocks,
  conflict fragments, or abrupt EOFs,
- files whose current state does not match the most likely intended edit.

Do not treat every modified file as broken.
Do not widen the candidate set without evidence.

### 4. Integrity Verification

Validate each candidate with file-type-appropriate checks where possible.

Checks may include:
- Rust, TOML, JSON, Markdown, and YAML structural sanity,
- unmatched delimiters or fences,
- malformed frontmatter,
- broken tables,
- truncated blocks,
- partial diff patterns,
- abrupt EOF clues,
- mismatch between current content and the most likely intended edit inferred
  from reconstructed context.

Rules:
- Validate whether suspicious files are actually corrupted.
- Classify based on evidence, not on the mere existence of modifications.
- Preserve user and prior-agent edits unless corruption is supported by evidence.

### 5. Repair Decision

Classify each candidate file as exactly one of:
- `clean`
- `corrupted and safe to repair automatically`
- `suspicious but requires human approval`
- `unrecoverable from available evidence`

Decision rules:
- `clean`: modified but structurally and contextually consistent.
- `corrupted and safe to repair automatically`: evidence strongly supports a
  narrow fix and the intended content is recoverable with high confidence.
- `suspicious but requires human approval`: the file may be damaged, but intent
  is uncertain or more than one repair is plausible.
- `unrecoverable from available evidence`: corruption appears real, but the
  intended content cannot be reconstructed safely.

### 6. Repair Execution

Repair only files classified as `corrupted and safe to repair automatically`.

Repair rules:
- prefer the smallest evidence-backed edit,
- preserve surrounding user and prior-agent work,
- never discard uncertain edits silently,
- do not broaden the repair into unrelated cleanup,
- record why each repair is considered safe.

If any candidate is in the `suspicious but requires human approval` class,
present the proposed repair in draft form first and wait for explicit approval.

### 7. Post-Repair Validation

Re-run the same integrity checks on every repaired file.

Also run narrow validation relevant to the interrupted task when appropriate:
- local syntax or parse checks,
- file-format validation,
- targeted diff review against reconstructed intent.

If validation fails, stop and report the file as unresolved.
Do not continue the original task.

### 8. Task Continuation

Resume the original task only after:
- trigger confirmation is complete,
- context reconstruction is documented,
- suspicious files have been scanned,
- integrity verification is complete,
- required repairs are complete,
- post-repair validation passes.

If recovery cannot establish a trustworthy baseline, halt and ask the human for
direction instead of continuing implementation.

## Conversation Output Order

When presenting drafts after clarification, use this exact order:

1. `Assumptions and Verified Constraints`
2. `Draft: .agents/skills/session-recovery/SKILL.md`
3. `Draft: .agents/workflows/workflow-session-recovery.md`
4. `Draft: pack/<agent_id>_<timestamp>/session_recovery_design_note.md`
5. `Approval Request`

Each draft must be copy-pasteable as markdown.

## Approval Gate

Before writing any file, ask for explicit approval in plain language.

Silence, partial feedback, or implied preference is not approval.

For any repair that is not clearly safe and minimal, show the draft repair or
proposed diff first and wait for approval before writing.

## Allowed Writes After Approval

If and only if the human explicitly approves, this skill may write:
- `.agents/skills/session-recovery/SKILL.md`
- `.agents/workflows/workflow-session-recovery.md`
- `pack/<agent_id>_<timestamp>/session_recovery_design_note.md`

No other writes are allowed unless the human explicitly expands scope.

## Retirement Procedure

If the running agent reaches 14 tool calls before completing the task:

1. stop doing new design work,
2. read `.agents/qa/model_routing_table.md` before composing the handoff,
3. write `pack/<agent_id>_<timestamp>/context.md`,
4. write `pack/<agent_id>_<timestamp>/handoff_prompt.md`,
5. present the handoff prompt in conversation as a fenced markdown block,
6. terminate the session immediately after handoff.

## Pack File Requirements

`context.md` must include:
- task status,
- files read,
- tool-call count,
- verified constraints,
- human answers received so far,
- draft status,
- open decisions,
- exact next step for the successor.

## Handoff Prompt Schema

Use this schema exactly:

```md
# Handoff Prompt

Role: <successor role>
Domain: <successor domain>
Model: <copied from .agents/qa/model_routing_table.md for the chosen successor role>
Task: Continue the session-recovery skill design task without restarting discovery.

## Read First
1. AGENTS.md
2. bug_pool/BUG_POOL.md
3. pack/<retiring_agent_id>_<timestamp>/context.md
4. .agents/qa/model_routing_table.md
5. Any still-relevant source files already identified in the pack

## Current State
- Status: <what is done>
- Drafts: <not started | partial | ready for approval>
- Human clarifications received: <list>
- Remaining blocker: <single main blocker or "none">

## Constraints
- No code writing during this task
- No modifications to knowledge/, coordinators/, AGENTS.md, ROOT_COORDINATOR.md, or .agents/qa/
- Draft-first in conversation, await approval before writing
- Preserve repo-specific rules while keeping wording IDE-portable
- The future skill must recover prior context, repair corruption safely, then continue the interrupted task

## Next Step
<one concrete next action>

## Deliverables
- Draft .agents/skills/session-recovery/SKILL.md
- Draft .agents/workflows/workflow-session-recovery.md
- Draft pack design note
```

## Operating Reminder

Design only.
No code.
No protected-file edits.
Clarify first.
Draft in conversation.
Wait for approval.
Write only approved files.
Retire cleanly if the tool budget is reached.
