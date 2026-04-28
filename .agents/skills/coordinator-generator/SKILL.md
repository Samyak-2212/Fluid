---
name: coordinator-generator
description: >
  Generates a spec-compliant coordinator PROMPT.md, a first-session
  handoff_prompt.md, and a bootstrap context.md stub for a new Fluid
  framework coordinator. Activate only in a fresh session dedicated to
  defining a new coordinator. Do not activate during implementation,
  debugging, or any active coordinator task.
---

# coordinator-generator Skill

## Activation

Activate this skill **only** by opening a fresh session and following the
`workflow-coordinator-generator` workflow. Do not invoke mid-session, during implementation
work, or while any coordinator task is active.

## Purpose

Produce three artifacts that define and bootstrap a new Fluid framework coordinator:

1. `coordinators/<name>/PROMPT.md` — the coordinator's permanent specification
2. `pack/<name>_<timestamp>/handoff_prompt.md` — the activation artifact for the
   coordinator's first session
3. `pack/<name>_<timestamp>/context.md` — a bootstrap stub documenting session 1 state

Additionally, produce an **IDE-portable version** of `PROMPT.md` in conversation only.
This version is not written to disk.

## Scope

- Defining a new top-level coordinator (a new CX slot in the hierarchy)
- Defining a new sub-coordinator within an existing coordinator's domain
- Generating all three output files listed above
- Producing an IDE-portable PROMPT.md variant in conversation

## Non-Scope

- Implementing any code
- Generating coordinator prompts for any existing running coordinator
- Modifying `knowledge/`, existing `coordinators/*/PROMPT.md` files, `AGENTS.md`,
  `ROOT_COORDINATOR.md`, `bug_pool/BUG_POOL.md`, or any `.agents/qa/` file
- Writing to `knowledge_b/`
- Proposing changes to `knowledge/dependency_graph.md` or `knowledge/project_manifest.md`
  directly — flag proposed additions as `[UNVERIFIED]` in conversation only
- Runtime tier switching or multi-tier feature flag design

## Required Pre-Read Files

Read in this exact order before asking any questions or producing any output:

1. `AGENTS.md`
2. `bug_pool/BUG_POOL.md`
3. `knowledge/project_manifest.md`
4. `knowledge/dependency_graph.md`
5. `knowledge/model_tier_policy.md`
6. `knowledge/file_structure.md`
7. `knowledge/capability_tiers.md`
8. `.agents/qa/model_routing_table.md`
9. `ROOT_COORDINATOR.md` (read-only — coordinator naming conventions only)
10. Any existing `pack/<name>*/context.md` for the target coordinator slot if one exists

After reading: identify all open bugs in `BUG_POOL.md` relevant to the target
component domain. Note them — they inform the new coordinator's responsibilities section.

## Coordinator Type Distinction

Before producing any questions, present the human with these two options and ask which
applies:

**Option A — New top-level coordinator**
A new CX slot in the project hierarchy. Requires:
- A new gate signal name (format: `[CX_COMPLETE]` or `[CX_INTERFACES_PUBLISHED]`)
- A wave assignment in `knowledge/dependency_graph.md`
- An entry in `knowledge/project_manifest.md`
- A role assignment in `.agents/qa/model_routing_table.md`
These additions are proposed as `[UNVERIFIED]` blocks in conversation only — they
are NOT written to `knowledge/` or `.agents/qa/` by this skill.

**Option B — Sub-coordinator within an existing domain**
A coordinator nested under an existing CX coordinator. Requires:
- A coordinator-local gate signal (naming convention: `[CX_<SUBNAME>_COMPLETE]`)
  [UNVERIFIED — no explicit sub-coordinator example exists in repo; verify against
  ROOT_COORDINATOR.md before finalising]
- File ownership scoped within the parent coordinator's domain
- The parent coordinator's PROMPT.md would need updating to delegate this sub-scope
  (flagged as a proposed change — this skill does not edit the parent PROMPT.md)

The human must confirm which applies before the question phase begins. Do not assume.

## Human Question Phase — Round 1

After coordinator type is confirmed, ask the following questions.
Present as a numbered list. Wait for all answers before proceeding.

### Fixed Core Questions (always asked, regardless of type)

1. **Name** — What is the coordinator's identifier (e.g. "C8", "C4-Constraints")?
   What is the human-readable domain name?
2. **Domain ownership** — Which files and directories does this coordinator own
   exclusively? List precisely (e.g. `physics_core/src/constraints/`).
3. **Gate signal(s)** — What completion signal(s) must this coordinator publish?
   Proposed name must follow `[CX_COMPLETE]` or `[CX_<NAME>_COMPLETE]` convention.
4. **Dependencies** — What existing gate signals must be published before this
   coordinator may begin? Cross-reference `knowledge/dependency_graph.md`.
5. **Unblocks** — Which other coordinators begin after this coordinator's gate signal
   is published?
6. **Model tier** — Which model tier(s) does this coordinator's work require?
   Which phases are Tier A? Which phases are Tier B?
   Reference `knowledge/model_tier_policy.md` and `.agents/qa/model_routing_table.md`.
7. **Completion gate checklist** — List all files that must exist and be non-empty
   before the gate signal is published.
8. **Needs-review scope** — Which output files, if any, must be tagged
   `[NEEDS_REVIEW: claude]` per repo policy?

### Type-Specific Follow-Up Questions

Ask only the follow-ups that apply to the confirmed coordinator type and domain.
Do not ask irrelevant follow-ups.

**Physics / numerical domain:**
- What numerical precision guarantees are required? (reference `knowledge/physics_contract.md`)
- Are unsafe blocks expected in this coordinator's output?
- What integrator type(s) will be implemented? (explicit Euler, RK4, implicit, other)
- Is CUDA or ROCm FFI required? If yes, which tier?

**Rendering domain:**
- Is wgpu pipeline setup required? If yes, which phases?
- What GPU capability tiers are targeted? (reference `knowledge/capability_tiers.md`)
- Are SPIR-V shaders authored by this coordinator or consumed from another?

**Build / config domain:**
- Which TOML configuration schemas does this coordinator own?
- Which feature flags does this coordinator define?
- What platform targets must be verified?

**Debugger / tooling domain:**
- Is an HTTP server embedded?
- What log format and directory convention apply?
- Does this coordinator integrate with the existing `debugger/logs/active/` path?

**New top-level coordinator (Option A only):**
- What build wave does this coordinator belong to?
  (Wave 0 = parallel from day 0; Wave 1 = after C1; etc.)
- What is the proposed coordinator identifier? Confirm it does not conflict with
  existing slots in `knowledge/project_manifest.md`.
- Propose a `model_routing_table.md` role entry — which model, and why?

## Round 1 Validation

After receiving Round 1 answers, validate before producing any draft:

- Gate signal name must not duplicate any existing signal in `knowledge/project_manifest.md`.
- File ownership must not overlap with any existing coordinator's domain
  (verified from `knowledge/file_structure.md` and existing `coordinators/*/PROMPT.md`).
- Model tier assignment must reference a role in `model_routing_table.md` or propose
  a new entry (flagged `[UNVERIFIED]` in conversation).
- If any answer is insufficient to fill a required PROMPT.md section: re-ask that
  specific item. Do not speculate. Do not proceed with incomplete information.
- If a conflict is found: report it precisely. Do not proceed until the human resolves it.

## Round 1.5 — Structured Outline (Approval Gate 1)

After Round 1 answers pass validation, produce a structured outline in conversation.
Do not produce full draft content yet.

The outline must include:

```
Coordinator: <name>
Type: <top-level | sub-coordinator>
Domain: <crate/directory>
Owns: <file list>
Gate signals: <list>
Blocked on: <list of prerequisite signals, or "none">
Unblocks: <list, or "none">
Model: <Tier A model> for <phases>; <Tier B model> for <phases>
Needs-review scope: <files, or "none">
Completion gate checklist: <file list>
Outputs to be written:
  - coordinators/<name>/PROMPT.md
  - pack/<name>_<timestamp>/handoff_prompt.md
  - pack/<name>_<timestamp>/context.md (stub)
Conversation-only:
  - IDE-portable PROMPT.md variant
```

Ask: **"Does this outline accurately represent the new coordinator? Approve to proceed
to full draft."**

Do not produce full drafts until the human explicitly approves the outline.

## Round 2 — Full Draft (Approval Gate 2)

After outline approval, produce all drafts in conversation in this order:

1. `coordinators/<name>/PROMPT.md` (Fluid-canonical)
2. `pack/<name>_<timestamp>/handoff_prompt.md`
3. `pack/<name>_<timestamp>/context.md` (bootstrap stub)
4. IDE-portable `PROMPT.md` variant (conversation only)

Each draft must be presented as a fenced markdown block.

Ask: **"Approve these drafts to write files to disk?"**

Do not write any file until the human explicitly approves.

## PROMPT.md Content Specification (Fluid-Canonical)

Every generated `coordinators/<name>/PROMPT.md` must include these sections in order.
Do not add or remove top-level headings.

```
# <Coordinator Name> — <Domain Name> Coordinator PROMPT

## Identity
## Domain
## Mandatory Reading (in this exact order, before any action)
## Responsibilities
## <Name> Completion Gate
## Implementation Constraints
## Cargo.toml for <crate>/ (if applicable)
## Sustainability Rules (excerpt — read AGENTS.md for full list)
## Model Tier for <Name> Work
## Output Checklist Before Gate
```

Content rules:

- `## Mandatory Reading` must list: `knowledge/dependency_graph.md`,
  `knowledge/capability_tiers.md`, `knowledge/model_tier_policy.md`,
  `bug_pool/BUG_POOL.md`, and the coordinator's own pack context if one exists.
  Add domain-specific knowledge files as warranted by Round 1 answers.
- `## Responsibilities` must list only files this coordinator owns exclusively.
  Do not list files owned by other coordinators.
- `## Completion Gate` must specify: all required files (name + required content),
  the gate signal to publish, the hard retirement trigger clause, and the pack file
  + handoff prompt requirement.
- `## Implementation Constraints` must not speculate. Include only constraints
  explicitly provided in Round 1 answers or directly verifiable from existing
  knowledge files.
- `## Model Tier` must reference `knowledge/model_tier_policy.md` and
  `.agents/qa/model_routing_table.md` by name. Approved Tier A models for this
  repository are Claude Sonnet and GPT-5.4/5.5. Do not hardcode model names without
  citing the routing table.
- `## Output Checklist Before Gate` must list every file in the completion gate
  checklist plus: pack file written, handoff prompt written and presented to user.
- Tag any section where information is incomplete or inferred as `[UNVERIFIED]`.
- Tag any item requiring human verification before acting as `[UNRESOLVED]`.

## IDE-Portable PROMPT.md Variant Specification

The IDE-portable version differs from the Fluid-canonical version only in these ways:

- Replace all absolute repo paths with relative path conventions (`./knowledge/`,
  `./bug_pool/`, etc.)
- Replace `"read AGENTS.md for full list"` with an inline summary of the three most
  critical sustainability rules for this coordinator's domain.
- Replace pack-directory references with `./pack/<coordinator-name>_<timestamp>/`.
- Replace gate signal mechanics with a comment block:
  `<!-- Gate signal: publish <SIGNAL_NAME> to project manifest per repo convention.
  Hard retirement trigger — see repo AGENTS.md. -->`
- All domain-specific content (domain ownership, responsibilities, implementation
  constraints, model tiers, completion gate checklist) remains identical to the
  Fluid-canonical version.

The IDE-portable version is presented in conversation and not written to disk.

## Handoff Prompt Specification

The generated `pack/<name>_<timestamp>/handoff_prompt.md` must follow this schema:

```md
# Handoff Prompt

Role: <coordinator role name>
Domain: <domain description>
Model: <model name from .agents/qa/model_routing_table.md>
Task: Begin <coordinator name> implementation from session 1.

## Read First
1. AGENTS.md
2. bug_pool/BUG_POOL.md
3. coordinators/<name>/PROMPT.md
4. knowledge/project_manifest.md
5. knowledge/dependency_graph.md
6. pack/<name>_<timestamp>/context.md
7. <any domain-specific knowledge files listed in PROMPT.md Mandatory Reading>

## Current State
- Status: Session 1. No prior work. Bootstrap context only.
- Drafts: N/A
- Blockers: <list of prerequisite gate signals not yet published, or "none">

## Constraints
- <top 3 constraints from PROMPT.md most critical for session 1>
- Do not write to knowledge/, coordinators/ (other than own domain), AGENTS.md,
  ROOT_COORDINATOR.md
- Draft-first in conversation, await approval before writing

## Next Step
<single concrete first action>

## Deliverables
<gate completion checklist copied verbatim from PROMPT.md>
```

The `Model:` field must be copied verbatim from `.agents/qa/model_routing_table.md`
for the matching role. Approved Tier A models: Claude Sonnet, GPT-5.4/5.5.
If the role does not yet exist in the routing table, write:
`[UNRESOLVED — add entry to .agents/qa/model_routing_table.md before activating
this coordinator]` and flag it in the output summary.

## Bootstrap Context.md Specification

The generated `pack/<name>_<timestamp>/context.md` stub must contain:

```md
# Context — <Coordinator Name> Session 1 Bootstrap

Agent ID: <name>_<timestamp>
Task status: NOT_STARTED
Files read: none
Tool-call count: 0
Prior work: none — this is session 1
Verified constraints: none yet
Human answers received: N/A
Draft status: N/A
Open decisions: <list any [UNRESOLVED] items from Round 1 answers, or "none">
Next step: Read all mandatory files listed in coordinators/<name>/PROMPT.md,
           then read this file's Open Decisions section, then proceed.
```

This file is a structural bootstrap stub, not a real session record. A future retiring
agent will overwrite it with actual session state.

## Writing Phase

After explicit human approval of all Round 2 drafts, write in this order:

1. `coordinators/<name>/PROMPT.md`
2. `pack/<name>_<timestamp>/handoff_prompt.md`
3. `pack/<name>_<timestamp>/context.md`

After writing, report:
- Exact path and byte count for each file written
- Any `[UNRESOLVED]` items requiring follow-up action by the human
- The handoff prompt text the human should paste to activate the new coordinator

Do not write any other files. Do not write the IDE-portable variant to disk.

## Failure Handling

- **Insufficient answers**: Re-ask specific missing items. Do not speculate.
  Do not proceed until answered.
- **Ownership conflict**: Report conflicting coordinator and owned files precisely.
  Halt until human resolves.
- **Gate signal conflict**: Report existing signal name. Propose two alternatives
  following naming convention. Halt until human selects one.
- **Missing routing table entry**: Write `[UNRESOLVED]` in handoff prompt Model field.
  Report in output summary. Do not block file writing for this alone.
- **Missing pre-read file**: Report missing file name and halt. All listed pre-read
  files must be present before the skill proceeds.
- **Human rejects outline or draft**: Take corrections and reproduce the rejected
  artifact. Do not write until explicit approval is given.

## Retirement and Handoff

If 14 tool calls are reached before the writing phase completes:

1. Stop all design and writing work immediately.
2. Read `.agents/qa/model_routing_table.md` to confirm successor model.
3. Write `pack/<agent_id>_<timestamp>/context.md` with current task state.
4. Write `pack/<agent_id>_<timestamp>/handoff_prompt.md` using the AGENTS.md schema.
5. Present the handoff prompt as a fenced markdown block in conversation.
6. Terminate. Do not continue work in the same session.

Successor role: Coordinator Generator Skill Agent
Successor model: Claude Sonnet or GPT-5.4/5.5 (Tier A — per
`.agents/qa/model_routing_table.md`; use whichever is available)

## Protected Files — Never Write

Under any circumstances, this skill must not write to:

- `knowledge/` (any file)
- Any existing `coordinators/*/PROMPT.md` (existing files only — new coordinator
  PROMPT.md files are allowed)
- `AGENTS.md`
- `ROOT_COORDINATOR.md`
- `bug_pool/BUG_POOL.md` (except appending a new bug entry using the BUG_POOL schema)
- `.agents/qa/` (any file)
- `knowledge_b/` (any file)

Proposals that would modify these files must be flagged to the human as `[UNVERIFIED]`
items in conversation. They require a separate Tier A task to action.
