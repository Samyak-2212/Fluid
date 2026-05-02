<!-- version: 14 -->
# File Structure

Last updated by: c8_session4_20260502T (C8 session 4 — wgpu viewport integration)
Reflects: app/src/viewport/ fully implemented (camera.rs, pipeline.rs, mod.rs); app/Cargo.toml updated (bytemuck, advanced). Merged onto version: 13.

## Root

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `AGENTS.md` | file | Root | active | Agent rules all agents read |
| `CLAUDE.md` | file | Root | active | Claude Code project memory; defers to AGENTS.md |
| `ROOT_COORDINATOR.md` | file | Root | active | User source prompt, not repo-authored output |
| `Cargo.toml` | file | Root | active | Workspace skeleton and shared dependency versions |
| `Cargo.lock` | file | Root | active | Cargo dependency lock file |
| `LICENSE` | file | Root | active | MIT OR Apache-2.0 licence |
| `.gitignore` | file | Root | active | Excludes build and log artifacts |
| `.codex/` | dir | Root | active | Codex IDE tooling directory |
| `target/` | dir | Build | gitignored | Cargo build cache |
| `README.md` | file | docs | draft | Workspace overview and build entry points |
| `USAGE.md` | file | docs | draft | Workspace usage reference |

## knowledge/

All files in `knowledge/` are Tier A authored and maintained. Files carry a `<!-- version: N -->` header and must be incremented on every write.

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `knowledge/capability_tiers.md` | file | Root | active | version: 2 (oneAPI resolved: infeasible) |
| `knowledge/physics_contract.md` | file | Root | active | version: 1 |
| `knowledge/dependency_graph.md` | file | Root | active | version: 2 |
| `knowledge/model_tier_policy.md` | file | Root | active | version: 1 |
| `knowledge/config_schema.md` | file | Root | active | version: 4 |
| `knowledge/file_structure.md` | file | Root/C2/C5/C8 | active | This file version: 11 |
| `knowledge/project_manifest.md` | file | Root | active | version: 24, C8 in-progress |

## bug_pool/

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `bug_pool/BUG_POOL.md` | file | All | active | Central bug tracker |

## coordinators/

All `PROMPT.md` files are Tier A only. Documentation work reads them for crate intent but does not modify them.

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `coordinators/core/PROMPT.md` | file | Root | active | C1 specification |
| `coordinators/build_system/PROMPT.md` | file | Root | active | C2 specification |
| `coordinators/rendering/PROMPT.md` | file | Root | active | C3 specification |
| `coordinators/physics_core/PROMPT.md` | file | Root | active | C4 specification |
| `coordinators/sim_components/PROMPT.md` | file | Root | active | C5 specification |
| `coordinators/sim_components/*/PROMPT.md` | file | C5 | active | Sub-coordinator prompts (7 total) |
| `coordinators/debugger/PROMPT.md` | file | Root | active | C6 specification |
| `coordinators/quality_gate/PROMPT.md` | file | Root | active | C7 specification |
| `coordinators/app/PROMPT.md` | file | Root | active | C8 specification |
| `coordinators/app/*/PROMPT.md` | file | C8 | planned | Sub-coordinator prompts (6 planned: C8-UI, C8-Viewport, C8-FileFormat, C8-Import, C8-SimBridge, C8-Assets) |
| `coordinators/agent_debugger/PROMPT.md` | file | C8 | planned | C9 specification (authored by C8 at [C8_INTERFACES_PUBLISHED]) |

## Crate Directories

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `core/` | dir | C1 | complete | ECS, units, math, time, threading, event bus |
| `physics_core/` | dir | C4 | **complete** | integrators, collision (GJK/EPA), constraints, rigid_body implemented; 22 tests pass |
| `rendering/` | dir | C3 | **complete** | wgpu init, Tier 0 CPU rasterizer, scene renderer, HTTP preview; 12 tests pass |
| `builder/` | dir | C2 | complete | Native build UI |
| `debugger/` | dir | C6 | **complete** | Localhost debugger surface (port 8081) |
| `app/` | dir | C8 | **in-progress** | Fluid GUI app — Session 4: ViewportProgram+Primitive wgpu shader implemented; orbit/pan/zoom camera; grid floor pipeline; cargo check EXIT:0. |
| `app/src/viewport/camera.rs` | file | C8 | **active** | Orbit camera: spherical coords, view_proj(), orbit/pan/zoom methods |
| `app/src/viewport/pipeline.rs` | file | C8 | **active** | [NEEDS_REVIEW: claude] ViewportPipelineState: wgpu LineList grid + PointList entity pipelines, mapped_at_creation upload |
| `app/src/viewport/mod.rs` | file | C8 | **active** | [NEEDS_REVIEW: claude] ViewportProgram (Program trait), ViewportPrimitive (Primitive trait), DragState, ViewportInteractState |
| `agent_debugger/` | dir | C9 | **in-progress** | Agent test harness — screenshot, widget control, protocol conformance |
| `components/fluid_simulator/` | dir | C5 | **implemented** | SPH (Wendland C2 + XSPH + Leap-Frog) + CFD (Chorin projection) + GPU FFI trait |
| `components/aerodynamic_simulator/` | dir | C5 | **implemented** | Thin-aerofoil lift/drag model (C_L, C_D polar) |
| `components/motion_force_simulator/` | dir | C5 | **implemented** | Gravity, spring-damper, hydraulic actuator, electric motor, joints |
| `components/thermodynamic_simulator/` | dir | C5 | **implemented** | Lumped capacitance + RK4 + Strang operator splitting |
| `components/fem_structural/` | dir | C5 | **implemented** | Euler-Bernoulli beam, Newmark-Beta dynamic solver, faer dense LU |

## Documentation Files

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `core/README.md` | file | docs | draft | Verified against C1-complete source |
| `core/USAGE.md` | file | docs | draft | Public API and config reference |
| `rendering/README.md` | file | docs | draft | Partial implementation and review markers noted |
| `rendering/USAGE.md` | file | docs | draft | Public API, config, and preview usage |
| `components/fluid_simulator/README.md` | file | docs | draft | Stub crate and gate-blocked notes |
| `components/fluid_simulator/USAGE.md` | file | docs | draft | No current public API |
| `components/aerodynamic_simulator/README.md` | file | docs | draft | Stub crate and gate-blocked notes |
| `components/aerodynamic_simulator/USAGE.md` | file | docs | draft | No current public API |
| `components/motion_force_simulator/README.md` | file | docs | draft | Stub crate and gate-blocked notes |
| `components/motion_force_simulator/USAGE.md` | file | docs | draft | No current public API |
| `components/thermodynamic_simulator/README.md` | file | docs | draft | Minimal `init()` API documented |
| `components/thermodynamic_simulator/USAGE.md` | file | docs | draft | Minimal `init()` API documented |
| `components/fem_structural/README.md` | file | docs | draft | Minimal `init()` API documented |
| `components/fem_structural/USAGE.md` | file | docs | draft | Minimal `init()` API documented |

## builder/

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `builder/Cargo.toml` | file | C2 | active | Build UI manifest |
| `builder/src/main.rs` | file | C2 | active | App entry point |
| `builder/src/subprocess.rs` | file | C2 | active | Cargo process control |
| `builder/src/config.rs` | file | C2 | active | Typed TOML loading |
| `builder/src/state.rs` | file | C2 | active | Build-state tracking |
| `builder/src/ui/` | dir | C2 | active | UI panels and widgets |

## config/

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `config/builder_flags.toml` | file | C2 | active | Builder flag schema |
| `config/core.toml` | file | C1 | active | Timestep and Rayon defaults |
| `config/rendering.toml` | file | C3 | **active** | preview_http_port, frame, camera, debug_overlay keys |
| `config/physics_core.toml` | file | C4 | **active** | constraint_solver_iterations=10, broadphase_cell_size=1.0 |
| `config/debugger.toml` | file | C6 | **active** | Debugger config (http port, logs) |
| `config/fluid_simulator.toml` | file | C5 | active | Created by C5 |
| `config/aerodynamic_simulator.toml` | file | C5 | active | Created by C5 |
| `config/thermodynamic_simulator.toml` | file | C5 | active | Created by C5 |
| `config/fem_structural.toml` | file | C5 | active | Created by C5 |
| `config/motion_force_simulator.toml` | file | C5 | active | Created by C5 |
| `config/app.toml` | file | C8 | planned | debug_server_port, debug_server_token, log_level |
| `config/component_manifest.toml` | file | C8 | planned | Component plugin registry |
| `config/agent_debugger.toml` | file | C9 | **active** | host, port, token, timeout_secs, screenshot_min_interval_ms |

## pack/

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `pack/root_coordinator_20260427T032847Z/` | dir | Root | active | Root coordinator context |
| `pack/c1_complete_20260428T080200Z/` | dir | C1 | active | C1 final session context |
| `pack/c2_complete_20260429T173700Z/` | dir | C2 | active | C2 final session context |
| `pack/codex_generate_readme_20260429T0211120530/` | dir | docs | active | Documentation-session pack snapshot |
| `pack/c3_rendering_20260428T173700Z/` | dir | C3 | active | C3 session context and handoff |
| `pack/c4_physics_core_20260429T230621Z/` | dir | C4 | active | C4 interfaces-published session context and handoff |
| `pack/c5_scaffold_20260429T213456Z/` | dir | C5 | active | C5 scaffold session context |
| `pack/c5_impl_20260429T214423Z/` | dir | C5 | active | C5 implementation checkpoint (soft retire) |
| `pack/c5_complete_20260430T064800Z/` | dir | C5 | active | C5 gate-verified completion pack |
| `pack/c8/` | dir | C8 | planned | C8 session pack dir (LATEST.md + MANIFEST.md) |
| `pack/c9/` | dir | C9 | **active** | C9 session pack dir (LATEST.md + MANIFEST.md) |

## knowledge_b/

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `knowledge_b/` | dir | Tier B | active | Proposal and review-area files |
| `knowledge_b/PROPOSED_doc_status_manifest_section.md` | file | docs | active | version: 2, all target crate docs marked DRAFT |

## out/

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `out/` | dir | Build | gitignored | Debug and release binaries |
| `out/debug/bin/` | dir | Build | gitignored | Debug executables per component |
| `out/release/bin/` | dir | Build | gitignored | Release executables per component |

## Stale Entry Policy

Mark outdated entries `[STALE: reason]` rather than leaving them incorrect.
