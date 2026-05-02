//! [NEEDS_REVIEW: claude]
//! GPU pipeline resources for the 3D viewport.
//!
//! Uses `iced::widget::shader::wgpu` re-exports — same device as iced backend.
//! This wgpu is the version bundled with iced 0.13 (wgpu ~0.19/0.20 API).
//! Do NOT mix with the standalone wgpu 29.0.1 types.
//!
//! `ViewportPipelineState` is stored in the iced `Storage` map and persists
//! across frames. It is created lazily on the first `prepare()` call.
//!
//! # Pipeline overview
//!
//! Two render pipelines share one uniform bind group (camera VP matrix):
//!
//! 1. **Grid pipeline** — `LineList` topology, draws the XZ floor grid.
//! 2. **Point pipeline** — `PointList` topology, draws entity-origin markers.
//!
//! Vertex format (both pipelines):
//! ```
//! struct Vertex { position: [f32; 3], color: [f32; 4] }
//! ```
//!
//! Uniform:
//! ```
//! struct Uniforms { view_proj: [[f32; 4]; 4] }
//! ```

use std::mem;
use iced::widget::shader::wgpu;

// ── Grid geometry ─────────────────────────────────────────────────────────────

/// Half-extent of the grid (metres).
const GRID_HALF: f32 = 10.0;
/// Grid spacing (metres).
const GRID_STEP: f32 = 1.0;
/// Lines per axis = 2 * HALF / STEP + 1.
const GRID_LINES_PER_AXIS: usize =
    (2.0 * GRID_HALF / GRID_STEP) as usize + 1;

// ── Placeholder sphere vertices ───────────────────────────────────────────────

/// One point per entity, max 256 before we stop drawing (session 5 lifts this).
pub const MAX_SPHERE_POINTS: u32 = 256;

// ── Vertex / Uniform POD types ────────────────────────────────────────────────

/// A single vertex with position and RGBA colour.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color:    [f32; 4],
}

/// Camera uniform (view-projection matrix, column-major).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniforms {
    pub view_proj: [[f32; 4]; 4],
}

// ── WGSL shaders ─────────────────────────────────────────────────────────────

const SHADER_SRC: &str = r#"
struct CameraUniforms {
    view_proj : mat4x4<f32>,
}

struct VertexInput {
    @location(0) position : vec3<f32>,
    @location(1) color    : vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_pos : vec4<f32>,
    @location(0)       color    : vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera : CameraUniforms;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_pos = camera.view_proj * vec4<f32>(in.position, 1.0);
    out.color    = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

// ── ViewportPipelineState ─────────────────────────────────────────────────────

/// All GPU resources for the viewport, stored in iced `Storage`.
///
/// Lazily initialised once on first prepare(). Persists across frames.
pub struct ViewportPipelineState {
    pub grid_pipeline:    wgpu::RenderPipeline,
    pub point_pipeline:   wgpu::RenderPipeline,
    pub grid_vertex_buf:  wgpu::Buffer,
    pub point_vertex_buf: wgpu::Buffer,
    pub uniform_buf:      wgpu::Buffer,
    pub uniform_bg:       wgpu::BindGroup,
    pub point_count:      u32,
}

impl ViewportPipelineState {
    /// Construct all GPU resources. Called once from `prepare()`.
    ///
    /// # Safety
    /// [NEEDS_REVIEW: claude] — raw wgpu device calls; bytemuck Pod casts.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // ── Shader module ────────────────────────────────────────────────
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("viewport_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        // ── Bind group layout (uniform buffer) ───────────────────────────
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   Some("viewport_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding:    0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty:                 wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size:   wgpu::BufferSize::new(
                        mem::size_of::<CameraUniforms>() as u64,
                    ),
                },
                count: None,
            }],
        });

        // ── Pipeline layout ──────────────────────────────────────────────
        let pll = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label:                Some("viewport_pll"),
            bind_group_layouts:   &[&bgl],
            push_constant_ranges: &[],
        });

        // ── Vertex attributes (shared by both pipelines) ─────────────────
        let vertex_attrs = [
            wgpu::VertexAttribute {
                offset:          0,
                shader_location: 0,
                format:          wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset:          mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format:          wgpu::VertexFormat::Float32x4,
            },
        ];
        let vertex_stride = mem::size_of::<Vertex>() as wgpu::BufferAddress;

        // ── Grid pipeline (LineList) ─────────────────────────────────────
        let grid_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label:  Some("viewport_grid_pipeline"),
            layout: Some(&pll),
            vertex: wgpu::VertexState {
                module:      &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_stride,
                    step_mode:    wgpu::VertexStepMode::Vertex,
                    attributes:   &vertex_attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module:      &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend:      Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology:           wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face:         wgpu::FrontFace::Ccw,
                cull_mode:          None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample:   wgpu::MultisampleState::default(),
            multiview:     None,
        });

        // ── Point pipeline (PointList) ───────────────────────────────────
        let point_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label:  Some("viewport_point_pipeline"),
            layout: Some(&pll),
            vertex: wgpu::VertexState {
                module:      &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_stride,
                    step_mode:    wgpu::VertexStepMode::Vertex,
                    attributes:   &vertex_attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module:      &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend:      Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology:           wgpu::PrimitiveTopology::PointList,
                strip_index_format: None,
                front_face:         wgpu::FrontFace::Ccw,
                cull_mode:          None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample:   wgpu::MultisampleState::default(),
            multiview:     None,
        });

        // ── Uniform buffer ───────────────────────────────────────────────
        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("viewport_uniform_buf"),
            size:               mem::size_of::<CameraUniforms>() as wgpu::BufferAddress,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ── Bind group ───────────────────────────────────────────────────
        let uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("viewport_uniform_bg"),
            layout:  &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding:  0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        // ── Grid vertex buffer (mapped_at_creation upload) ───────────────
        // [NEEDS_REVIEW: claude] — mapped_at_creation used for synchronous upload.
        let grid_verts = build_grid_vertices();
        let grid_vbuf_size =
            (mem::size_of::<Vertex>() * grid_verts.len()) as wgpu::BufferAddress;
        let grid_vertex_buf = {
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label:              Some("viewport_grid_vbuf"),
                size:               grid_vbuf_size,
                usage:              wgpu::BufferUsages::VERTEX,
                mapped_at_creation: true,
            });
            {
                // SAFETY: Vertex is Pod; cast to bytes for upload.
                let mut view = buf.slice(..).get_mapped_range_mut();
                view.copy_from_slice(bytemuck::cast_slice::<Vertex, u8>(&grid_verts));
            }
            buf.unmap();
            buf
        };

        // ── Point vertex buffer (max capacity, written each frame) ───────
        let point_vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("viewport_point_vbuf"),
            size:               (mem::size_of::<Vertex>() * MAX_SPHERE_POINTS as usize)
                as wgpu::BufferAddress,
            usage:              wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            grid_pipeline,
            point_pipeline,
            grid_vertex_buf,
            point_vertex_buf,
            uniform_buf,
            uniform_bg,
            point_count: 0,
        }
    }
}

// ── Grid geometry builder ─────────────────────────────────────────────────────

/// Build line-list vertices for the XZ floor grid.
///
/// Grid spans [-GRID_HALF .. +GRID_HALF] in both X and Z.
/// Minor lines: dim grey (#2d2d3d). Every 5th line: slightly brighter.
pub fn build_grid_vertices() -> Vec<Vertex> {
    let steps = GRID_LINES_PER_AXIS as i32;
    let cap   = (steps as usize) * 4 + 4; // 2 verts per line × 2 axes + 4 axis verts
    let mut verts = Vec::with_capacity(cap);

    let half = GRID_HALF;
    let step = GRID_STEP;

    for i in 0..steps {
        let t   = -half + (i as f32) * step;
        let major = i % 5 == 0;
        let a   = if major { 0.35_f32 } else { 0.18_f32 };
        let col = [0x2d as f32 / 255.0, 0x2d as f32 / 255.0, 0x3d as f32 / 255.0, a];

        // Line along Z at x = t
        verts.push(Vertex { position: [t, 0.0, -half], color: col });
        verts.push(Vertex { position: [t, 0.0,  half], color: col });

        // Line along X at z = t
        verts.push(Vertex { position: [-half, 0.0, t], color: col });
        verts.push(Vertex { position: [ half, 0.0, t], color: col });
    }

    // Axis lines (X = red, Z = blue) — slightly above floor to avoid z-fighting.
    let axis_y = 0.001;
    verts.push(Vertex { position: [-half, axis_y, 0.0], color: [0.5, 0.1, 0.1, 0.7] });
    verts.push(Vertex { position: [ half, axis_y, 0.0], color: [0.8, 0.2, 0.2, 0.7] });
    verts.push(Vertex { position: [0.0, axis_y, -half], color: [0.1, 0.1, 0.5, 0.7] });
    verts.push(Vertex { position: [0.0, axis_y,  half], color: [0.2, 0.2, 0.8, 0.7] });

    verts
}
