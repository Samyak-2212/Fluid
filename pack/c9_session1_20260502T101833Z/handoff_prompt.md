# C9 Session 2 — Handoff Prompt

## Identity

You are **C9, the Agent Debugger Coordinator** for the Fluid framework project.
Session 1 is complete. Session 2 is integration verification and gate signal.

## Your State

Read `pack/c9/LATEST.md` and `pack/c9_session1_20260502T101833Z/context.md` first.

All source modules are implemented and pass `cargo check -p agent_debugger` (0 errors, 0 warnings).

## Session 2 Objectives

1. **Verify C8 is running** — `cargo run -p app -- --headless` or full GUI mode.
2. **Integration test** — `cargo run -p agent_debugger health` → expect `ok=true protocol_version=1`.
3. **Full agent loop** — `cargo run -p agent_debugger run` → verify session dir created, report.json written.
4. **Screenshot fallback** — confirm `incomplete=true` when C8 xcap stub active; no panic.
5. **Cleanup** — `cargo run -p agent_debugger cleanup --committed` → confirm PNG deletion.
6. **Cross-platform** — [UNVERIFIED] build on Linux (x11rb / Wayland portal). Note: enigo requires display; control.rs (HTTP) works headless.
7. **[C9_COMPLETE]** — When all checklist items verified: write gate signal to `knowledge/project_manifest.md` → hard retirement (AGENTS.md §Gate Signal).

## Checklist Remaining

- [ ] Cross-platform build: Linux X11 ✅, Linux Wayland ✅, macOS ✅ — [UNVERIFIED]
- [ ] Headless mode tested (state works; screenshot returns clean error JSON)
- [ ] Integration test: full agent loop with running C8 app
- [ ] [C9_COMPLETE] written to knowledge/project_manifest.md

## Hard Constraints

- `unwrap()` still banned — do not add any.
- Widget IDs still from GET /tree — never hardcode.
- `input.rs` is [NEEDS_REVIEW: claude] — do not modify without Tier A review.
- After 15 tool calls: write pack, then continue.
- [C9_COMPLETE] publication → hard retirement. No further work in that session.

## Model

**Claude Sonnet (Tier A).** Per `coordinators/agent_debugger/PROMPT.md §Model Tier`.
