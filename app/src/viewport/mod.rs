//! [NEEDS_REVIEW: claude]
//! 3D viewport — `iced::widget::shader` + wgpu render pass (DEC-002).
//!
//! # Architecture
//!
//! ```text
//! FluidApp.viewport: ViewportState          ← scene snapshot (entity_count etc.)
//!       │
//!       └─ view_viewport_panel()
//!             ├─ constructs ViewportProgram  ← Program<AppMessage> (lightweight)
//!             └─ iced::widget::shader(program)
//!                       │
//!                       ├─ State = ViewportInteractState  ← camera, drag (per-widget)
//!                       ├─ update() — processes mouse events, updates camera
//!                       ├─ draw()   — produces ViewportPrimitive (scene snapshot)
//!                       └─ Primitive::prepare / render — GPU work
//! ```
//!
//! # No second wgpu Device
//! The `Device` and `Queue` arrive via `Primitive::prepare()` — they are the
//! iced wgpu backend's device. Do NOT call `Instance::new()` anywhere.
//!
//! # Camera rules
//! - Left-mouse drag  → orbit (azimuth / elevation)
//! - Right-mouse drag → pan (target translation)
//! - Scroll wheel     → zoom (radius)
//! Camera state lives in `ViewportInteractState` (the Program's associated State).

pub mod camera;
pub mod pipeline;

use std::mem;

use glam::Mat4;
use iced::{
    Rectangle, mouse,
    advanced::Shell,
    widget::{
        canvas::event::Status,
        shader::{self, wgpu, Event, Storage, Viewport},
    },
};

use crate::app::AppMessage;

use camera::Camera;
use pipeline::{
    CameraUniforms, Vertex, ViewportPipelineState,
    MAX_SPHERE_POINTS,
};

// ── ViewportState ─────────────────────────────────────────────────────────────

/// Application-level viewport state stored in `FluidApp`.
///
/// Used to pass scene information into the per-frame `ViewportProgram`.
#[derive(Debug, Default)]
pub struct ViewportState {
    /// Number of ECS entities in the scene (read-only snapshot).
    pub entity_count: u64,
}

// ── ViewportProgram ───────────────────────────────────────────────────────────

/// Implements `Program<AppMessage>` for the 3D viewport shader widget.
///
/// Constructed cheaply in `view_viewport_panel()` each frame — just holds a
/// snapshot of scene data needed for rendering.
pub struct ViewportProgram {
    /// Number of entities to render as placeholder spheres.
    pub entity_count: u64,
}

// ── ViewportInteractState ─────────────────────────────────────────────────────

/// Per-widget interactive state, managed by the iced shader runtime.
///
/// Created via `Default`, persisted across frames.
#[derive(Debug)]
pub struct ViewportInteractState {
    /// Current orbit camera.
    pub camera: Camera,
    /// Current drag operation.
    pub drag: DragState,
}

impl Default for ViewportInteractState {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            drag:   DragState::Idle,
        }
    }
}

/// Which mouse button is currently held and what operation it drives.
#[derive(Debug, Clone, PartialEq)]
pub enum DragState {
    Idle,
    Orbiting { last: iced::Point },
    Panning  { last: iced::Point },
}

// ── ViewportPrimitive ─────────────────────────────────────────────────────────

/// Data produced by `Program::draw()` and consumed by the wgpu backend.
///
/// Carries everything `prepare()` and `render()` need — no references into
/// the iced widget tree are held here.
#[derive(Debug)]
pub struct ViewportPrimitive {
    /// Camera view-projection matrix (column-major, ready for the uniform).
    view_proj: Mat4,
    /// Number of entity points to draw (capped at MAX_SPHERE_POINTS).
    entity_count: u32,
}

// ── Program impl ──────────────────────────────────────────────────────────────

impl shader::Program<AppMessage> for ViewportProgram {
    type State     = ViewportInteractState;
    type Primitive = ViewportPrimitive;

    // Produce a primitive from the current camera state.
    fn draw(
        &self,
        state:  &ViewportInteractState,
        _cursor: mouse::Cursor,
        bounds:  Rectangle,
    ) -> ViewportPrimitive {
        let view_proj = state.camera.view_proj(bounds.width, bounds.height);
        let entity_count = (self.entity_count as u32).min(MAX_SPHERE_POINTS);
        ViewportPrimitive { view_proj, entity_count }
    }

    // Process mouse events → update camera orbit/pan/zoom.
    fn update(
        &self,
        state:  &mut ViewportInteractState,
        event:  Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        _shell: &mut Shell<'_, AppMessage>,
    ) -> (Status, Option<AppMessage>) {
        match event {
            // ── Button press ─────────────────────────────────────────────
            Event::Mouse(mouse::Event::ButtonPressed(btn)) => {
                if let Some(pos) = _cursor.position() {
                    match btn {
                        mouse::Button::Left => {
                            state.drag = DragState::Orbiting { last: pos };
                            return (Status::Captured, None);
                        }
                        mouse::Button::Right => {
                            state.drag = DragState::Panning { last: pos };
                            return (Status::Captured, None);
                        }
                        _ => {}
                    }
                }
            }

            // ── Button release ───────────────────────────────────────────
            Event::Mouse(mouse::Event::ButtonReleased(_)) => {
                if state.drag != DragState::Idle {
                    state.drag = DragState::Idle;
                    return (Status::Captured, None);
                }
            }

            // ── Cursor moved ─────────────────────────────────────────────
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                match &state.drag.clone() {
                    DragState::Orbiting { last } => {
                        let dx = position.x - last.x;
                        let dy = position.y - last.y;
                        state.camera.orbit(dx, dy);
                        state.drag = DragState::Orbiting { last: position };
                        return (Status::Captured, None);
                    }
                    DragState::Panning { last } => {
                        let dx = position.x - last.x;
                        let dy = position.y - last.y;
                        state.camera.pan(dx, dy);
                        state.drag = DragState::Panning { last: position };
                        return (Status::Captured, None);
                    }
                    DragState::Idle => {}
                }
            }

            // ── Scroll / zoom ─────────────────────────────────────────────
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let amount = match delta {
                    mouse::ScrollDelta::Lines { y, .. }  => y,
                    mouse::ScrollDelta::Pixels { y, .. } => y / 20.0,
                };
                state.camera.zoom(amount);
                return (Status::Captured, None);
            }

            _ => {}
        }

        (Status::Ignored, None)
    }

    fn mouse_interaction(
        &self,
        state:  &ViewportInteractState,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match &state.drag {
            DragState::Orbiting { .. } => mouse::Interaction::Grab,
            DragState::Panning  { .. } => mouse::Interaction::ResizingHorizontally,
            DragState::Idle           => mouse::Interaction::Crosshair,
        }
    }
}

// ── Primitive impl ────────────────────────────────────────────────────────────

/// Clear colour matching the Fluid dark theme (#0f0f13).
const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0x0f as f64 / 255.0,
    g: 0x0f as f64 / 255.0,
    b: 0x13 as f64 / 255.0,
    a: 1.0,
};




impl shader::Primitive for ViewportPrimitive {
    /// [NEEDS_REVIEW: claude]
    /// Lazily creates GPU pipelines on first call, then updates the uniform
    /// and entity point buffers each frame.
    fn prepare(
        &self,
        device:   &wgpu::Device,
        queue:    &wgpu::Queue,
        format:   wgpu::TextureFormat,
        storage:  &mut Storage,
        _bounds:  &Rectangle,
        _viewport: &Viewport,
    ) {
        // ── Lazy pipeline creation ────────────────────────────────────────
        if !storage.has::<ViewportPipelineState>() {
            storage.store(ViewportPipelineState::new(device, format));
        }

        let ps = storage.get_mut::<ViewportPipelineState>()
            // SAFETY: we just stored it above; this branch always has a value.
            .expect("ViewportPipelineState missing after store");

        // ── Update camera uniform buffer ──────────────────────────────────
        let cols = self.view_proj.to_cols_array_2d();
        let uniforms = CameraUniforms { view_proj: cols };
        queue.write_buffer(&ps.uniform_buf, 0, bytemuck::bytes_of(&uniforms));

        // ── Update entity point buffer ────────────────────────────────────
        let count = self.entity_count;
        ps.point_count = count;

        if count > 0 {
            // Place placeholder sphere markers at the origin (session 5 will
            // query actual entity positions from the ECS world).
            let sphere_color = [0.388, 0.400, 0.945, 0.9_f32]; // #6366f1
            let verts: Vec<Vertex> = (0..count)
                .map(|_i| Vertex {
                    position: [0.0, 0.0, 0.0],
                    color:    sphere_color,
                })
                .collect();
            let bytes = bytemuck::cast_slice::<Vertex, u8>(&verts);
            queue.write_buffer(&ps.point_vertex_buf, 0, bytes);
        }
    }

    /// [NEEDS_REVIEW: claude]
    /// Encodes the render pass: clear → grid lines → entity points.
    fn render(
        &self,
        encoder:     &mut wgpu::CommandEncoder,
        storage:     &Storage,
        target:      &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let ps = match storage.get::<ViewportPipelineState>() {
            Some(ps) => ps,
            // Pipeline not yet initialised — skip silently.
            None => return,
        };

        // Convert clip_bounds to scissor rect using our shim.
        let scissor = clip_to_scissor_rect(clip_bounds);

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("viewport_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view:           target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load:  wgpu::LoadOp::Clear(CLEAR_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes:         None,
            occlusion_query_set:      None,
        });

        // Scissor to the widget's clip region.
        pass.set_scissor_rect(
            scissor.x,
            scissor.y,
            scissor.width,
            scissor.height,
        );

        // ── Draw grid ────────────────────────────────────────────────────
        // Grid vertex buffer includes axis lines appended after the grid lines.
        let total_grid_verts = ps.grid_vertex_buf.size() as u32
            / mem::size_of::<Vertex>() as u32;

        pass.set_pipeline(&ps.grid_pipeline);
        pass.set_bind_group(0, &ps.uniform_bg, &[]);
        pass.set_vertex_buffer(0, ps.grid_vertex_buf.slice(..));
        pass.draw(0..total_grid_verts, 0..1);

        // ── Draw entity origin markers ────────────────────────────────────
        if ps.point_count > 0 {
            pass.set_pipeline(&ps.point_pipeline);
            pass.set_bind_group(0, &ps.uniform_bg, &[]);
            pass.set_vertex_buffer(0, ps.point_vertex_buf.slice(..));
            pass.draw(0..ps.point_count, 0..1);
        }
    }
}

// ── wgpu::util helper ─────────────────────────────────────────────────────────

/// Convert a `Rectangle<u32>` clip region to scissor rect components.
mod wgpu_util_shim {
    pub struct ScissorRect {
        pub x:      u32,
        pub y:      u32,
        pub width:  u32,
        pub height: u32,
    }

    pub fn clip_to_scissor_rect(r: &iced::Rectangle<u32>) -> ScissorRect {
        ScissorRect {
            x:      r.x,
            y:      r.y,
            width:  r.width.max(1),
            height: r.height.max(1),
        }
    }
}

use wgpu_util_shim::clip_to_scissor_rect;
