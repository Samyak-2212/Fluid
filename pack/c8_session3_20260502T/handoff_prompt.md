C8 Session 3 → Session 4 Handoff Prompt
You Are: C8, the Fluid GUI Application Coordinator — session 4.

Model: Claude Sonnet (Tier A)

Mandatory Reading Order:

coordinators/app/PROMPT.md
pack/c8/LATEST.md
pack/c8_session3_20260502T/context.md
app/DECISIONS.md
app/INTERFACES.md
bug_pool/BUG_POOL.md

Context: Sessions 1–3 complete. cargo check -p app is EXIT:0. [C8_INTERFACES_PUBLISHED] is published (manifest v25). C9 is unblocked.

Session 4 Work — C8-Viewport:

Replace the placeholder container in Panel::Viewport3D (view_viewport_panel in app/src/app.rs) with a real iced::widget::shader integration.
Implement app/src/viewport/mod.rs — wgpu render pass using iced::widget::shader.
The shader widget receives a wgpu::Device + wgpu::Queue via the Program trait (iced::widget::shader::Program).
Implement a minimal wgpu pipeline: clear to #0f0f13, draw a grid floor plane (simple vertex buffer, line topology).
Camera orbit/pan/zoom — respond to mouse events forwarded from the pane_grid; camera state lives in ViewportState.
Scene read-only query each frame: count entities, draw placeholder spheres at origin.
No raycasting selection yet — that is session 5.
cargo check -p app must pass before ending session.
Write pack + present Session 5 handoff (C8-Scene: raycasting selection + gizmos).

Key constraints:
- Box<dyn WorldAny> — read-only each frame, no direct mutation from viewport.
- wgpu = 29.0.1 — must match rendering/ crate version.
- iced = 0.13.1 with wgpu feature — iced::widget::shader::Program is the integration point.
- every unsafe {} needs [NEEDS_REVIEW: claude].
- The iced shader widget runs on the iced wgpu backend — do NOT create a second wgpu Device/Instance. Access the backend's device via the Program::draw(frame: &mut shader::Frame<'_, Primitive>) interface.

Important context from session 3:
- FluidApp.viewport: ViewportState is a stub (camera: Camera, all fields zero).
- Panel::Viewport3D calls self.view_viewport_panel() which returns a placeholder container.
- The pane_grid feeds resize events — the viewport must respond to PaneResized to update its surface dimensions.
- AppMessage::Noop is available for any Task::none() paths.

Blocked by nothing. Begin immediately.
