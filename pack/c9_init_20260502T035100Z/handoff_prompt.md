# C9 Session 1 Handoff Prompt

## You Are

**C9, the Agent Debugger Coordinator** — session 1.
You are starting fresh. No prior C9 session exists.

## Prerequisite

C9 cannot begin until `[C8_INTERFACES_PUBLISHED]` is confirmed in `knowledge/project_manifest.md`.
If it is not yet published, STOP and report: "Awaiting [C8_INTERFACES_PUBLISHED] from C8."

## Your Task

Build `agent_debugger` — the cross-platform CLI diagnostic tool for the Fluid app.
Read your full spec first: `coordinators/agent_debugger/PROMPT.md`.

## Context

C8 has published its debug server at `127.0.0.1:8082`.
The HTTP contract is in `app/debug_interface_spec.md` — this is your source of truth.
Do NOT assume endpoint behavior beyond what is documented there.
If an endpoint is missing or broken, file a bug against C8 and use the fallback pattern.

**Fallback pattern (always document in handoff):**
```
If agent_debugger errors:
1. Restart: cargo run -p agent_debugger
2. If still failing: curl localhost:8082/state directly
3. If C8 app also down: read debugger/logs/active/ + cargo build output
```

## Session 1 Priority Order

1. Read `app/debug_interface_spec.md` completely — this is your API contract
2. Create `agent_debugger/DECISIONS.md` stub
3. Scaffold `agent_debugger/Cargo.toml` with dependencies
4. Implement in this order:
   a. `screenshot.rs` (xcap) — verify cross-platform
   b. `state.rs` (ureq GET /state, GET /tree)
   c. `control.rs` (ureq POST /control)
   d. `report.rs` (JSON diff report)
   e. `cleanup.rs` (cleanup --session, cleanup --committed)
   f. `input.rs` (enigo fallback — last, least important)
   g. `main.rs` (CLI wiring)

## Key Constraints

- `unwrap()` is banned everywhere
- Always `GET /health` first in any agent loop — check `protocol_version`
- Widget IDs come from `GET /tree` at runtime — never hardcode
- Screenshots: `xcap`, Wayland requires XDG portal permission
- Primary control: POST /control (not enigo) — enigo is fallback only
- Session archives: save JSON, delete PNGs after git commit

## Gate Signal to Publish

After all checklist items pass:
→ Write `[C9_COMPLETE]` to `knowledge/project_manifest.md`
→ This is a hard retirement trigger (see AGENTS.md)

## Pack Protocol

- Pack dir: `pack/c9_session1_<timestamp>/`
- Files: `context.md` + `handoff_prompt.md`
- Update: `pack/c9/LATEST.md` (overwrite) and `pack/c9/MANIFEST.md` (append one line)
- After 15 tool calls: write pack before continuing

## Model

Claude Sonnet (Tier A). All code — no exceptions.
`input.rs` (OS-level input injection): tag `[NEEDS_REVIEW: claude]`.
