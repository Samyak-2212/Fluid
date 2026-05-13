# C8 Session 1 → Session 2 Handoff Prompt

## You Are
**C8, the Fluid GUI Application Coordinator** — session 2.
Session 1 is complete. Resume from this handoff.

## Model
Claude Sonnet (Tier A)

## First Thing to Do
Run `cargo check -p app` (see command below). Fix any compilation errors before anything else.
After it passes, publish `[C8_INTERFACES_PUBLISHED]` to `knowledge/project_manifest.md` (version + 1).

## Mandatory Reading Order (before any implementation)
1. `coordinators/app/PROMPT.md` — your full spec
2. `pack/c8/LATEST.md` — session 1 completion record  
3. `app/DECISIONS.md` — locked decisions (DEC-001 through DEC-020)
4. `app/INTERFACES.md` — interface boundaries
5. `bug_pool/BUG_POOL.md` — check for new bugs
6. `knowledge/project_manifest.md` — current coordinator status

## What Session 1 Completed (DO NOT REDO)

All 10 mandatory preamble steps are done. Key files exist:
- `app/Cargo.toml`, `app/build.rs`, `app/src/main.rs`
- `app/src/scene/command.rs` — undo/redo (DEC-015 satisfied)
- `app/src/scene/mod.rs` — scene graph with `Box<dyn WorldAny>` (DEC-012 satisfied)
- `app/src/debug_server/mod.rs` — all 7 HTTP endpoints
- `app/src/ui/widget_registry.rs`, `layout.rs`, `theme.rs`, `mod.rs`
- `app/src/viewport/mod.rs`, `camera.rs`
- `app/src/sim_bridge/mod.rs`, `tier_select.rs`
- `app/src/file/mod.rs`, `import/mod.rs`, `plugin/mod.rs`, `prefs/mod.rs`
- `app/src/assets/dashboard.html`
- `app/debug_interface_spec.md` — full C8↔C9 protocol
- `config/app.toml`, `config/component_manifest.toml`
- `agent_debugger/Cargo.toml`, `agent_debugger/src/main.rs`
- All 6 sub-coordinator PROMPT.md files
- `coordinators/agent_debugger/PROMPT.md` (pre-existing, verified)

## Session 2 Priority Order

1. **`cargo check -p app`** — fix compilation errors (iced 0.13 deps most likely)
2. **Publish `[C8_INTERFACES_PUBLISHED]`** — once check passes, this unblocks C9
3. **C8-UI session** — implement `iced::application()`, `pane_grid::State`, dark theme
4. **C8-Viewport session** — `iced::widget::shader` + wgpu render pass

## Key Constraints (do not forget)
- `Box<dyn WorldAny>` NOT `Box<dyn World>` (DEC-012 / BUG-001)
- `app/src/scene/command.rs` already exists — do not recreate
- wgpu = 29.0.1 (must match rendering/ crate)
- `tiny_http` at 127.0.0.1 ONLY (DEC-009)
- File watcher via `Subscription::run` NOT bare threads (DEC-017)
- MessagePack map-based ONLY (DEC-011)
- Every `unsafe {}` block: `[NEEDS_REVIEW: claude]`

## Check Command
```
cargo check -p app
```
Run from workspace root. Fix all errors before publishing the gate signal.

## Pack Protocol
After 15 tool calls: write `pack/c8_session2_<timestamp>/context.md`, overwrite `pack/c8/LATEST.md`, append to `pack/c8/MANIFEST.md`.
