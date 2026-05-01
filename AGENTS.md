<!-- version: 1 -->
# AGENTS.md

Canonical shared agent instructions for this repository.
If an IDE supports its own project instruction file or rule format, keep that file aligned
with this one rather than forking the workflow rules.

## Rules — no exceptions

- No greetings, sign-offs, or filler in any output file or commit message.
- State facts. If uncertain, write [UNVERIFIED] or [UNRESOLVED].
- Read your coordinator PROMPT.md first. Read knowledge/ second. Read knowledge_b/ third (skeptically). Then act.
- After more than 15 tool calls: write a pack file, then continue or hand off.
- When publishing any completion gate signal ([CX_INTERFACES_PUBLISHED] or
  [CX_COMPLETE]): write a final pack file and terminate the session immediately.
  Do not continue work in the same session after a gate signal. This is hard
  retirement — not optional.
- Update knowledge/file_structure.md after touching more than 3 files.
- Check bug_pool/BUG_POOL.md before starting — your bug may already exist.
- Tag any output touching physics, rendering, unsafe blocks, or CUDA/ROCm FFI
  as [NEEDS_REVIEW: claude] if produced by a Tier B model.
- Tier B models: never modify knowledge/, coordinators/*/PROMPT.md, or ROOT_COORDINATOR.md.
  File proposed changes in bug_pool under Prompt/Knowledge Changes.
- knowledge/ files carry a `<!-- version: N -->` header. Increment N on every write.
  Read the current version before writing — if it changed since your session start,
  merge your changes onto the current file; do not overwrite.

- Before writing any handoff prompt, read .agents/qa/model_routing_table.md
  for the correct Model: field.
- Before hard retirement, read .agents/qa/tier_a_commit_protocol.md and
  execute the commit procedure before writing the pack file.

## Where to find your task

1. coordinators/<your_domain>/PROMPT.md — your specification
2. knowledge/dependency_graph.md — what you are blocked on
3. knowledge/capability_tiers.md — hardware targets for every feature
4. knowledge/physics_contract.md — numerical accuracy requirements
5. knowledge/model_tier_policy.md — which model should do which work
6. bug_pool/BUG_POOL.md — open bugs in your domain
7. pack/<relevant_id>/context.md — prior agent progress on this task

## Model discipline

- Tier B models: implement from spec, do not redesign interfaces.
- Tier B models on critical code: add [NEEDS_REVIEW: claude] file header.
- Claude and GPT-5.4 (Tier A): work from the C7 batched review queue, not ad-hoc requests.
- Never use Claude for boilerplate, test scaffolding, or config file generation.

## Build commands

- cargo build                                        (debug, default)
- cargo build --release                              (optimized — do not use for iteration)
- FLUID_TIER=N cargo build --features <component>   (explicit tier and component)
- Component selection via native builder UI (builder/) or CLI flags
- All available flags defined in config/builder_flags.toml
- Output: out/debug/ or out/release/, binaries in out/<mode>/bin/<component>
- Tier is compile-time only — changing tier requires a full recompile. Do not
  implement runtime tier switching.

## Debugger

- Debugger runs an embedded HTTP server on localhost.
- Open the debugger from any IDE preview surface or any local browser that can reach localhost.
- In IDX/Antigravity, the preview pane auto-detects that localhost server when available.
- Logs go to debugger/logs/active/ — never delete, archive on bug close.

## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- For cross-module "how does X relate to Y" questions, prefer `graphify query "<question>"`, `graphify path "<A>" "<B>"`, or `graphify explain "<concept>"` over grep — these traverse the graph's EXTRACTED + INFERRED edges instead of scanning files
- After modifying code files in this session, run `graphify update .` to keep the graph current (AST-only, no API cost)
