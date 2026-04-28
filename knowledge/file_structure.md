<!-- version: 3 -->
# File Structure

Last updated by: codex_generate_readme (session: codex_generate_readme_20260429T0211120530)
Reflects documentation files created for the workspace root, `core/`, `rendering/`, and component crates. Merged onto version: 2.

## Root

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `AGENTS.md` | file | Root | active | Agent rules all agents read |
| `ROOT_COORDINATOR.md` | file | Root | active | User source prompt, not repo-authored output |
| `Cargo.toml` | file | Root | active | Workspace skeleton and shared dependency versions |
| `.gitignore` | file | Root | active | Excludes build and log artifacts |
| `README.md` | file | docs | draft | Workspace overview and build entry points |
| `USAGE.md` | file | docs | draft | Workspace usage reference |

## knowledge/

All files in `knowledge/` are Tier A authored and maintained. Files carry a `<!-- version: N -->` header and must be incremented on every write.

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `knowledge/capability_tiers.md` | file | Root | active | version: 1 |
| `knowledge/physics_contract.md` | file | Root | active | version: 1 |
| `knowledge/dependency_graph.md` | file | Root | active | version: 1 |
| `knowledge/model_tier_policy.md` | file | Root | active | version: 1 |
| `knowledge/config_schema.md` | file | Root | active | version: 1 |
| `knowledge/file_structure.md` | file | Root/C2/docs | active | This file version: 3 |
| `knowledge/project_manifest.md` | file | Root | active | version: 4, C1 and C2 complete |

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
| `coordinators/debugger/PROMPT.md` | file | Root | active | C6 specification |
| `coordinators/quality_gate/PROMPT.md` | file | Root | active | C7 specification |

## Crate Directories

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `core/` | dir | C1 | complete | ECS, units, math, time, threading, event bus |
| `physics_core/` | dir | C4 | pending | Integrators, collision, constraints |
| `rendering/` | dir | C3 | in_progress | `wgpu`, Tier 0 framebuffer, scene renderer |
| `builder/` | dir | C2 | complete | Native build UI |
| `debugger/` | dir | C6 | pending | Localhost debugger surface |
| `components/fluid_simulator/` | dir | C5 | pending | SPH and CFD scaffold |
| `components/aerodynamic_simulator/` | dir | C5 | pending | Aerodynamics scaffold |
| `components/motion_force_simulator/` | dir | C5 | pending | Force and actuator scaffold |
| `components/thermodynamic_simulator/` | dir | C5 | pending | Minimal `init()` stub |
| `components/fem_structural/` | dir | C5 | pending | Minimal `init()` stub |

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
| `config/rendering.toml` | file | C3 | pending | Preview, frame, camera, overlay keys |
| `config/physics_core.toml` | file | C4 | pending | Solver and broadphase keys |
| `config/debugger.toml` | file | C6 | pending | Planned debugger config |
| `config/fluid_simulator.toml` | file | C5 | pending | Planned fluid config |
| `config/aerodynamic_simulator.toml` | file | C5 | pending | Planned aero config |
| `config/thermodynamic_simulator.toml` | file | C5 | pending | Planned thermal config |
| `config/fem_structural.toml` | file | C5 | pending | Planned FEM config |
| `config/motion_force_simulator.toml` | file | C5 | pending | Planned motion config |

## pack/

| Path | Type | Owner | Status | Notes |
|------|------|-------|--------|-------|
| `pack/root_coordinator_20260427T032847Z/` | dir | Root | active | Root coordinator context |
| `pack/c1_complete_20260428T080200Z/` | dir | C1 | active | C1 final session context |
| `pack/c2_complete_20260429T173700Z/` | dir | C2 | active | C2 final session context |
| `pack/codex_generate_readme_20260429T0211120530/` | dir | docs | active | Documentation-session pack snapshot |

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
