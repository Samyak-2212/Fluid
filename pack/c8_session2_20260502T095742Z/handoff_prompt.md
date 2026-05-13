# C8 Session 2 → Session 3 Handoff Prompt

## You Are
**C8, the Fluid GUI Application Coordinator** — session 3.
Sessions 1 and 2 are complete. Resume from this handoff.

## Model
Claude Sonnet (Tier A)

## First Thing to Do
Read mandatory files (order below), then begin C8-UI implementation.
No `cargo check` needed — session 2 already confirmed EXIT:0.

## Mandatory Reading Order (before any implementation)
1. `coordinators/app/PROMPT.md` — your full spec
2. `pack/c8/LATEST.md` — session 2 completion record
3. `app/DECISIONS.md` — locked decisions DEC-001 through DEC-020
4. `app/INTERFACES.md` — interface boundaries
5. `bug_pool/BUG_POOL.md` — check for new bugs
6. `knowledge/project_manifest.md` — current status (version 25)

6 reads. Then implement.

## What Sessions 1 & 2 Completed (DO NOT REDO)

### Session 1 — Skeleton
- `app/Cargo.toml`, `app/build.rs`, `app/src/main.rs` (headless + stub windowed mode)
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

### Session 2 — Compilation Fix + Gate
- Fixed `app/Cargo.toml`: `fbxcel-dom "0.9"` → `"0.0.10"` (only 0.0.x exists on crates.io)
- `cargo check -p app`: **EXIT:0, 0 errors**, 32 dead_code warnings (expected skeleton)
- **`[C8_INTERFACES_PUBLISHED]` published** to `knowledge/project_manifest.md` (v25)
- C9 is now UNBLOCKED

## Session 3 Work: C8-UI

Implement the Iced application wiring and panel layout. All code goes in `app/`.

### Step 1 — `app/src/main.rs`: wire `iced::application()`

Replace the TODO placeholder (line 131–140) with:

```rust
iced::application("Fluid", FluidApp::update, FluidApp::view)
    .theme(FluidApp::theme)
    .subscription(FluidApp::subscription)
    .run()
    .expect("Iced run failed");
```

Define `FluidApp` in a new `app/src/app.rs` (or inline in `main.rs` — your call).
`FluidApp` must hold:
- `pane_grid::State<Panel>` — the tiling layout state
- `Arc<RwLock<AppStateSnapshot>>` — written at each frame boundary
- `Arc<RwLock<WidgetRegistry>>` — owned here, Arc clone given to debug server
- `Arc<RwLock<Vec<PendingControl>>>` — polled from debug server
- `CommandHistory` — undo/redo stack
- `Option<Scene>` — active scene

### Step 2 — `app/src/ui/layout.rs`: pane_grid tiling

Implement `pane_grid::State<Panel>` with 5 panels:
```
┌──────────┬──────────────────────────────┬───────────────┐
│ Outliner │        3D Viewport           │  Properties   │
│          │   (iced::widget::shader)     │               │
├──────────┤                              ├───────────────┤
│ SimSetup │  Timeline / Playback         │  Results      │
└──────────┴──────────────────────────────┴───────────────┘
```
Use `iced::widget::pane_grid` — split/resize/drag enabled.
Every panel header must have `.id()` set (widget registry requirement).

### Step 3 — `app/src/ui/theme.rs`: dark professional theme

Implement custom `iced::Theme` via `iced::theme::Custom` or `iced::theme::Palette`.
Palette:
- Background: `#0f0f13`
- Surface:    `#1a1a24`
- Primary:    `#6366f1` (indigo-500)
- Secondary:  `#818cf8`
- Text:       `#e2e8f0`
- Danger:     `#f43f5e`

### Step 4 — `app/src/ui/mod.rs`: message type + view

Define `AppMessage` enum (at minimum):
- `PaneResized(pane_grid::ResizeEvent)`
- `PaneDragged(pane_grid::DragEvent)`
- `Tick` (for frame boundary state updates)
- `ControlReceived(ControlAction)` (from debug server poll)

Implement `view()` returning `iced::Element<AppMessage>`.
Implement `subscription()` returning time-based tick + any file watcher subscription.

### Step 5 — Viewport panel placeholder

For the 3D viewport panel, render a placeholder `iced::widget::container` with
centered text `"Viewport — C8-Viewport (session 4)"`.
Do NOT implement wgpu/shader yet — that is session 4 (C8-Viewport).

### Step 6 — `cargo check -p app` must pass

After all changes, run `cargo check -p app`. Fix all errors before ending the session.

## Key Constraints (never forget)
- `Box<dyn WorldAny>` NOT `Box<dyn World>` (DEC-012 / BUG-001)
- wgpu = 29.0.1 (must match `rendering/`)
- iced resolved to `0.13.1` — use only 0.13.x API
- `tiny_http` at 127.0.0.1 ONLY (DEC-009)
- File watcher via `Subscription::run` NOT bare threads (DEC-017)
- MessagePack map-based ONLY (DEC-011)
- Every `unsafe {}` block: `[NEEDS_REVIEW: claude]`
- Every interactive widget: `.id()` must be set (debug server contract)

## iced 0.13 API Notes
- `iced::application(title, update, view)` — functional API
- `pane_grid::State::new(first_pane_data)` + `State::split(axis, pane, data)`
- `pane_grid::ResizeEvent` / `pane_grid::DragEvent`
- `iced::widget::pane_grid(content_fn)` where content_fn: `|pane, content| -> pane_grid::Content<_>`
- Dark theme: construct via `iced::Theme::custom(name, palette)` using `iced::theme::Palette`

## Pack Protocol
After 15 tool calls: write `pack/c8_session3_<timestamp>/context.md`,
overwrite `pack/c8/LATEST.md`, append to `pack/c8/MANIFEST.md`.

## Session End Gate
After `cargo check -p app` passes with C8-UI wired:
- Write pack files (above)
- Update `knowledge/project_manifest.md` (increment version, update commit log)
- Present Session 4 handoff prompt to user (C8-Viewport)
- Do NOT publish `[C8_COMPLETE]` — that requires full completion checklist
