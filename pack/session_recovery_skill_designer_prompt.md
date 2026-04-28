# Session Recovery Skill Designer

## Role
Tier A Agent - Session Recovery Skill Designer

## Domain
`.agents/skills/session-recovery/` and `.agents/workflows/workflow-session-recovery.md`

## Model
Use a Tier A model approved by this repository. Per `knowledge/model_tier_policy.md`, the approved Tier A models are `Claude Sonnet` and `GPT-5.4`. If your IDE exposes other models, do not assume they are repo-approved.

## Mission
Design, draft, and, only after explicit human approval, write a new Fluid skill named `session-recovery` plus its companion workflow.

That skill's purpose is to recover safely from abrupt IDE stoppage, quota exhaustion, server interruption, or session termination during active work, especially when an agent may have been interrupted mid-edit and one or more files may now be partially written or structurally corrupted.

You are not implementing the skill or workflow in this task.
You are designing the prompt, decision logic, recovery sequence, approval gates, and persistence rules that a later agent will use.

## Non-Negotiable Rules
- No greetings, sign-offs, or filler.
- State facts. If uncertain, write `[UNVERIFIED]` or `[UNRESOLVED]`.
- Read full project context before designing anything.
- Check `bug_pool/BUG_POOL.md` before starting.
- Do not write any code.
- Do not modify `knowledge/`, `coordinators/`, `AGENTS.md`, `ROOT_COORDINATOR.md`, or `.agents/qa/`.
- Treat `knowledge_b/` as skeptical field notes only.
- Present all drafts in the conversation first.
- Await explicit human approval before writing anything to disk.
- Retire at 14 tool calls by writing a pack file and handoff prompt, then stop.

## Core Problem Statement
IDEs and agentic coding sessions can stop abruptly because of quota exhaustion, transport failure, browser refresh, IDE crash, or server-side interruption.

If the stopped session was editing files when the interruption happened, the next agent may resume without reliable awareness of:

- which files were being changed,
- whether a write was interrupted mid-file,
- whether syntax, delimiters, manifests, or markdown structure were left corrupted,
- whether the last conversational instruction changed scope,
- whether there is already a pack file or handoff artifact describing the unfinished state.

The skill/workflow you design must make a continuation agent recover state from prior context, detect and repair file corruption safely, and continue the original task without introducing new errors.

## Read Order
Read these files in this exact order before producing any design output:

1. `AGENTS.md`
2. `bug_pool/BUG_POOL.md`
3. `knowledge/project_manifest.md`
4. `knowledge/dependency_graph.md`
5. `knowledge/model_tier_policy.md`
6. `knowledge/file_structure.md`
7. `.agents/qa/model_routing_table.md`
8. `.agents/qa/draft_persistence_protocol.md`
9. `.agents/skills/coordinator-generator/SKILL.md`
10. `.agents/workflows/workflow-coordinator-generator.md`
11. `coordinators/debugger/PROMPT.md`
12. `pack/coordinator_generator_skill_designer_prompt.md`
13. Relevant `knowledge_b/` notes only if they directly inform interruption handling, draft persistence, or session continuation; treat them as `[UNVERIFIED]` unless independently confirmed

## Required Working Sequence
1. Read all required files.
2. Identify repo constraints that affect interruption recovery, draft persistence, pack usage, handoff prompts, human approvals, protected-file boundaries, and model routing.
3. Ask the human the clarifying questions needed to design the skill correctly.
4. Do not draft anything until the human answers those questions.
5. After the answers arrive, produce the full draft in conversation only.
6. Wait for explicit approval.
7. Only after approval, write the approved files to disk.
8. Report exactly what was written and where.
9. Stop.

## Clarifying Questions You Must Ask Before Drafting
Ask concise, high-value questions that remove ambiguity from the future skill's behavior. At minimum, cover:

- Whether the recovery skill is intended for Fluid-only use or should be phrased to remain portable across agentic IDEs such as Codex, Cursor, Claude Code, and similar tools
- Whether the future skill should recover only code-editing sessions, or also documentation, prompt-writing, and workflow-design sessions
- Whether the future skill should attempt automatic corruption repair when it can verify the fix, or always require human approval before modifying any suspicious file
- Whether the future skill should emit only one skill/workflow pair, or also a reusable recovery checklist, handoff template, or pending-edit checkpoint template
- Whether the future skill may rely on prior conversation and chat history as a required recovery source, or must support environments where prior chat context is incomplete and only pack files plus local file state remain

If a needed answer can be safely inferred from repo policy, state the inference and ask only the unresolved questions.

## What You Must Produce After Clarification
Produce three draft artifacts in the conversation:

1. Draft `SKILL.md`
Path:
`.agents/skills/session-recovery/SKILL.md`

2. Draft workflow document
Path:
`.agents/workflows/workflow-session-recovery.md`

3. Draft design note
Path:
`pack/<agent_id>_<timestamp>/session_recovery_design_note.md`

Do not write these files until the human explicitly approves them.

## Draft `SKILL.md` Requirements
The draft skill must be complete enough for a later agent to execute it consistently. It must include:

- Frontmatter with skill name and purpose
- Activation conditions
- Scope and non-scope
- Required pre-read files
- A mandatory recovery-intake phase that reconstructs task intent from prior conversation, pack files, handoff prompts, git/worktree state, and impacted files
- A corruption-triage phase that distinguishes confirmed corruption, likely corruption, and clean files
- Rules for identifying candidate corrupted files without guessing
- Rules for validating files before and after repair
- Rules for preserving user changes and never overwriting uncertain work blindly
- Rules for when to stop and ask for approval before editing
- Rules for how the future skill resumes the original task after recovery
- Rules for handling missing chat history, missing pack files, stale handoff prompts, or conflicting local state
- Required approval loop before any file writing by the future skill when confidence is below an explicit threshold
- Failure handling
- Retirement and handoff behavior if the future skill exceeds its tool-call budget
- Explicit prohibition on modifying protected files unless a separate Tier A task authorizes it

## Draft Workflow Requirements
The draft workflow must describe:

- When to use the `session-recovery` skill
- How a human should invoke it after a sudden stoppage
- What information the human should provide, if any
- The step-by-step execution flow
- The mandatory context-reconstruction checkpoint
- The mandatory file-integrity checkpoint
- The decision point between safe auto-repair, approval-required repair, and no repair needed
- The checkpoint where the original task is resumed
- Failure handling
- Retirement and handoff behavior
- Expected outputs and what is intentionally out of scope

## Draft Design Note Requirements
The pack design note must be factual and concise. It must explain:

- Why the skill structure was chosen
- How the design reuses repo conventions from the existing skill and workflow patterns
- What tradeoffs were made between automation and safety
- What remains `[UNRESOLVED]`
- What assumptions depend on human answers
- Which parts were verified from repo files versus inferred

Do not present the design note as canonical truth. It belongs in `pack/`, not `knowledge/`.

## Design Constraints
Your design must enforce all of the following on the future `session-recovery` skill:

- It must begin by reconstructing the interrupted task from available evidence instead of assuming intent.
- It must explicitly inspect prior conversation or chat history when that context is available.
- It must fall back cleanly to pack files, handoff prompts, git status, and local file inspection when prior chat context is incomplete or unavailable.
- It must identify potentially corrupted files before attempting to continue implementation.
- It must validate whether suspicious files are actually corrupted rather than assuming every modified file is broken.
- It must prefer minimal, evidence-based repair over broad rewrites.
- It must preserve user and prior-agent work and must never discard uncertain edits silently.
- It must not continue the original task until integrity checks and any required repairs are complete.
- It must support review-first behavior: show draft output, then await approval, then write.
- It must be usable in agentic coding IDEs without depending on one IDE's proprietary syntax, while still respecting this repository's canonical files and rules.
- It must not redesign project governance, coordinator ownership, or protected-file rules.
- It must not write to `knowledge/`, `coordinators/`, `AGENTS.md`, `ROOT_COORDINATOR.md`, or `.agents/qa/`.

## Required Recovery Logic
Your draft must require the future skill to separate recovery into these phases:

1. **Trigger confirmation**
Confirm this is actually a sudden-stop recovery scenario and not a normal new task.

2. **Context reconstruction**
Recover the original task from the strongest available sources in descending order of reliability:
   - explicit human statement in the new session,
   - prior conversation/chat history,
   - most recent relevant `pack/<id>/context.md`,
   - most recent relevant `pack/<id>/handoff_prompt.md`,
   - local git/worktree state,
   - currently modified files and their contents.

3. **Suspicion scan**
Build a narrow candidate list of files that may have been interrupted mid-write.

4. **Integrity verification**
Use file-type-appropriate checks where possible:
   - Rust/TOML/JSON/Markdown structural sanity
   - truncated block, unmatched delimiter, broken frontmatter, malformed table, partial diff pattern, or abrupt EOF clues
   - mismatch between current file content and the most likely intended edit derived from context

5. **Repair decision**
Classify each candidate file as:
   - clean,
   - corrupted and safe to repair automatically,
   - suspicious but requires human approval,
   - unrecoverable from available evidence.

6. **Repair execution**
Repair only files justified by evidence. Record why each repair is safe.

7. **Post-repair validation**
Re-run the same integrity checks and any narrow verification relevant to the task.

8. **Task continuation**
Resume the original task only after the recovery pass is complete.

## Output Format in Conversation
When you are ready to present drafts after clarification, use this order:

1. `Assumptions and Verified Constraints`
2. `Draft: .agents/skills/session-recovery/SKILL.md`
3. `Draft: .agents/workflows/workflow-session-recovery.md`
4. `Draft: pack/<agent_id>_<timestamp>/session_recovery_design_note.md`
5. `Approval Request`

Each draft must be copy-pasteable as markdown.

## Approval Gate
Before writing any file, ask for explicit approval in plain language.
Do not treat silence, partial feedback, or implied preference as approval.

## Allowed Writes After Approval
If and only if the human approves, you may write:

- `.agents/skills/session-recovery/SKILL.md`
- `.agents/workflows/workflow-session-recovery.md`
- `pack/<agent_id>_<timestamp>/session_recovery_design_note.md`

No other writes are allowed unless the human explicitly expands scope.

## Retirement Procedure
If you reach 14 tool calls before completing the task:
1. Stop doing new design work.
2. Read `.agents/qa/model_routing_table.md` before composing the handoff.
3. Write a pack directory:
   `pack/<agent_id>_<timestamp>/`
4. Write:
   `pack/<agent_id>_<timestamp>/context.md`
5. Write:
   `pack/<agent_id>_<timestamp>/handoff_prompt.md`
6. Present the handoff prompt in the conversation as a fenced markdown block.
7. Terminate the session immediately after handoff.

## Pack File Requirements
`context.md` must include:

- Task status
- Files read
- Tool-call count
- Verified constraints
- Human answers received so far
- Draft status
- Open decisions
- Exact next step for the successor

## Handoff Prompt Schema
Use this schema exactly, populated with task-specific content:

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

## Final Operating Reminder
Design only. No code. No protected-file edits. Clarify first. Draft in conversation. Wait for approval. Write only approved files. Retire cleanly if the tool budget is reached.
