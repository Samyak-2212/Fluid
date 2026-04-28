<!-- QA-AGENT-COMPLETE: issue-7 -->
# Tier A — Git Commit Protocol at Hard Retirement
<!-- version: 1 -->
<!-- Tier A owned. Increment version on every write.                      -->
<!-- Every Tier A agent MUST execute this protocol immediately before      -->
<!-- writing its retirement pack file and handoff prompt.                  -->
<!-- Do not edit without Tier A review.                                    -->

## When this applies

Every Tier A hard retirement — triggered by publishing a gate signal
(`[CX_INTERFACES_PUBLISHED]` or `[CX_COMPLETE]`) in
`knowledge/project_manifest.md`.

Soft retirement (15 tool calls) does NOT trigger a commit. Soft
retirement is mid-task. Only completed, gate-verified work is committed.

Tier B agents never commit. Git is Tier A territory only.

## Pre-commit verification

Before committing, confirm:

1. `cargo check --workspace` exits 0.
   If it does not, do not commit. File a `## High` bug and output the
   failure message from Issue 6's FAILURE PROTOCOL. Do not retire until
   the build is clean or the failure is explicitly handed to a repair agent.

2. The gate signal has been written to `knowledge/project_manifest.md`.
   Do not commit without the gate signal in place — the commit represents
   a verified gate, not just any working state.

## Commit procedure

Stage all modified and new files owned by your coordinator domain:

```bash
git add <your domain paths>
git add knowledge/project_manifest.md
git add knowledge/file_structure.md   # if touched this session
```

Do not stage:
- `knowledge_b/` entries — these are staging notes, not deliverables
- `/tmp/` files — never in git
- Pack files under `pack/` — committed separately only if needed for
  audit; default is do not stage

Commit with this exact message schema — no deviations:

```
[TIER_A_REVIEW] <coordinator-id>(<domain>): <one line description>

gate: <gate signal published this session>
agent: <agent_id>
timestamp: <ISO 8601>
```

Examples:
```
[TIER_A_REVIEW] C1(core): publish ECS traits and units module

gate: [C1_INTERFACES_PUBLISHED]
agent: c1-coordinator-20260428T091233Z
timestamp: 2026-04-28T09:12:33Z
```

```
[TIER_A_REVIEW] C2(builder): complete egui UI and subprocess management

gate: [C2_COMPLETE]
agent: c2-coordinator-20260428T143012Z
timestamp: 2026-04-28T14:30:12Z
```

## After committing — record the SHA

Immediately after the commit, run:
```bash
git rev-parse HEAD
```

Write the output SHA to `knowledge/project_manifest.md` under the
coordinator's entry, using this exact field:

```
Last clean checkpoint SHA: <sha>
```

If a prior `Last clean checkpoint SHA` field exists, replace it — only
the most recent clean checkpoint matters for rollback purposes.
Increment the `<!-- version: N -->` header on `project_manifest.md`.

## What this enables

Any subsequent agent — Tier A or the Issue 6 compiler check — can read
`Last clean checkpoint SHA` from `knowledge/project_manifest.md` and give the
human a precise revert target. The human runs:

```bash
git reset --hard <sha>
```

No agent needs to infer, guess, or search git log. The SHA is always
current and always written by a verified, gate-passing Tier A session.

## Tier B never commits

Tier B agents produce files but do not commit. Their output enters git
only when a subsequent Tier A agent runs this protocol after verifying
the build passes. If Tier B output breaks the build, the Tier A agent
never commits — the broken files are reset by the human using the last
clean SHA. From git's perspective, the broken Tier B session never
happened.
