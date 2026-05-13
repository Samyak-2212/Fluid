# C8 Session 1 Handoff Prompt

## You Are

**C8, the Fluid GUI Application Coordinator** — session 1.
You are starting fresh. No prior C8 session exists.

## Your Task

Build the Fluid native simulation application. Read your full spec first:
`coordinators/app/PROMPT.md` — follow the mandatory reading order listed there.

## Context

All upstream coordinators (C1–C7) are complete. Their published interfaces are available:
- `core/` (C1) — ECS `World`, `WorldAny`, units, event bus
- `rendering/` (C3) — `Camera`, `GpuContext`, `RenderSurface`
- `physics_core/` (C4) — integrators, collision, constraints
- `components/` (C5) — fluid, aero, thermo, FEM, motion simulators
- `debugger/` (C6) — log system at port 8081

Key constraint: use `Box<dyn WorldAny>` (NOT `Box<dyn World>`) for ECS access.
`World` is not dyn-compatible — see BUG-001 resolution in `core/src/ecs/traits.rs`.

## Session 1 Priority Order

**Do these first, in order, before any logic code:**

1. Add `"app"` and `"agent_debugger"` to root `Cargo.toml` workspace.members
2. `knowledge/dependency_graph.md` — add C8/C9 wave + signal entries (increment version)
3. `.agents/qa/model_routing_table.md` — add C8/C9 entries (increment version)
4. `knowledge/file_structure.md` — add app/ + agent_debugger/ sections (increment version)
5. `knowledge/config_schema.md` — add app.toml + component_manifest.toml schemas (increment version)
6. `knowledge/project_manifest.md` — add C8/C9 gate signal rows
7. Root `.gitignore` — add `app/bin/**` and `agent_debugger/sessions/**/*.png`
8. `app/DECISIONS.md` — DEC-001 through DEC-020 (copy from PROMPT.md)
9. `app/INTERFACES.md` — stub with planned interface boundaries
10. `app/research_dump/_INDEX.md` — stub

**Then begin implementation, starting with:**
- `app/src/scene/command.rs` — undo/redo command pattern MUST come before any scene mutation

## Architecture Highlights

- UI: `iced` 0.13+ with `pane_grid` tiling
- 3D: `iced::widget::shader` + wgpu (no Bevy runtime)
- Debug server: `tiny_http` at 127.0.0.1:8082 (same crate as C6's debugger)
- Widget registry: `app/src/ui/widget_registry.rs` — owned by C8-UI, shared Arc to debug server
- Hot-swap prefs: `notify` watcher via `iced::Subscription` (NOT bare threads)
- User prefs: `directories` crate → OS config dir (separate from `config/`)
- Sim execution: in-process Tier 0 (Rayon) + subprocess Tier 1–3 (pre-compiled binaries)

## Gate Signal to Publish

After skeleton + debug server + sub-coordinator PROMPT.md files are ready:
→ Write `[C8_INTERFACES_PUBLISHED]` to `knowledge/project_manifest.md`
→ This unblocks C9 (Agent Debugger)

## Pack Protocol

- Pack dir: `pack/c8_session1_<timestamp>/`
- Files: `context.md` + `handoff_prompt.md`
- Update: `pack/c8/LATEST.md` (overwrite) and `pack/c8/MANIFEST.md` (append one line)
- After 15 tool calls: write pack before continuing

## Model

Claude Sonnet (Tier A). All code — no exceptions.
