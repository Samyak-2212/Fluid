<!-- version: 13 -->
# Project Manifest

## Project: Fluid
Language: Rust (edition 2021+)
Root session: root_coordinator_20260427T032847Z
Status last updated: 2026-04-30T06:48:00+05:30
C3 Last clean checkpoint SHA: d00186b1b1619c22a85f1ed347ca650a055dd019
C4 Last clean checkpoint SHA: a4018a7aa5dd7d52baa3c0b77b8d9d1e11a6a276
C5 Last clean checkpoint SHA: d710739d168dd34844b8aa09529f8db98f7b9a59

---

## Coordinator Status

| Coordinator | Domain | Status | Gate Signal | Notes |
|-------------|--------|--------|-------------|-------|
| C1 — Core Systems | `core/` | COMPLETE | `[C1_INTERFACES_PUBLISHED]` ✅ `[C1_COMPLETE]` ✅ | All impl done; 26 tests pass |
| C2 — Build System | `builder/` | COMPLETE | `[C2_COMPLETE]` ✅ | cargo build -p builder: 0 errors, 0 warnings |
| C3 — Rendering | `rendering/` | COMPLETE | `[C3_COMPLETE]` ✅ | 12 tests pass; BUG-008/009 filed for C7 review |
| C4 — Physics Core | `physics_core/` | COMPLETE | `[C4_INTERFACES_PUBLISHED]` ✅ `[C4_COMPLETE]` ✅ | 22 tests pass; full impl complete |
| C5 — Sim Components | `components/` | **COMPLETE** | `[C5_COMPLETE]` ✅ | 26 tests pass; SPH/CFD/Aero/Thermo/FEM/Motion implemented |
| C6 — Debugger | `debugger/` | BLOCKED | `[C6_COMPLETE]` pending | Waiting on C1+C2 |
| C7 — Quality Gate | (cross-cutting) | NOT_STARTED | `[C7_COMPLETE]` pending | Can begin setup with C1+C2 |
| Root | (this file) | IN_PROGRESS | `[ROOT_COMPLETE]` pending | Writing final manifest |

---

## Completion Gate Signals

Signals written here by coordinators upon reaching their gates.
Writing any `[CX_INTERFACES_PUBLISHED]` or `[CX_COMPLETE]` signal is a hard retirement trigger.

<!-- Signals are appended below as coordinators complete their gates -->

[C1_INTERFACES_PUBLISHED]
Published by: C1 (session: c1_core_20260428T022400Z)
Timestamp: 2026-04-28T02:24:00+05:30
Gate files verified:
- core/src/units.rs — 13 SI newtype wrappers, all required types present, 8 unit tests pass
- core/src/ecs/traits.rs — EntityId, Component, World, System<W> defined; System uses W: World type param (not dyn World — World is not dyn-compatible due to generic methods; documented in trait)
- core/src/event_bus.rs — Event blanket impl + EventBus trait defined
cargo test -p core: 13 passed, 0 failed, EXIT:0

C3, C4, C6 may now begin.

---

[C1_COMPLETE]
Published by: C1 (session: c1_continuation_20260428T080200Z)
Timestamp: 2026-04-28T08:02:00+05:30
All C1 post-gate work verified:
- config/core.toml: timestep_seconds, max_ticks_per_frame, rayon_num_threads
- core/src/math/mod.rs: glam 0.32.1 verified, [UNVERIFIED] removed
- core/src/threading/traits.rs: rayon 1.12.0 verified, [UNVERIFIED] removed
- core/src/ecs/world.rs: ArchetypeWorld concrete impl, 8 tests pass [NEEDS_REVIEW: claude]
- core/src/event_bus_impl.rs: LocalEventBus concrete impl, 5 tests pass [NEEDS_REVIEW: claude]
cargo test -p core: 26 passed, 0 failed, EXIT:0
BUG-005 filed for C2 (workspace.edition warning)

C1 domain closed. File new core/ bugs to BUG_POOL.md, assign to C7 for triage.

---

[C2_COMPLETE]
Published by: C2 (session: c2_build_system_20260429T173700Z)
Timestamp: 2026-04-29T17:41:00+05:30
Gate files verified:
- builder/src/main.rs — FluidBuilderApp, eframe 0.34.1 fn ui + fn logic
- builder/src/subprocess.rs — BuildProcess: spawn, poll_output, kill, is_running, exit_status; child.kill() only
- builder/src/config.rs — FlagEntry/BuilderConfig/FlagState; typed TOML, no panic on missing keys
- builder/src/state.rs — ComponentStatus, BuildSessionState, BuildState
- builder/src/ui/ — flag_panel.rs, output_panel.rs, component_list.rs
- config/builder_flags.toml — 7 initial flags populated
cargo build -p builder: 0 errors, 0 warnings, Exit: 0
BUG-002 CLOSED. BUG-005 CLOSED. BUG-008 filed and closed (rendering wgpu gl→gles).

C6 (Debugger) and C7 (Quality Gate) may now begin.

---

[C3_COMPLETE]
Published by: C3 (session: c3_rendering_20260428T173700Z)
Timestamp: 2026-04-29T03:05:00+05:30
All C3 gate criteria verified:
- rendering/Cargo.toml: wgpu 29.0.1, softbuffer 0.4.8, winit 0.30.13, tiny_http 0.12.0, image 0.25.10 — all versions verified on docs.rs
- rendering/build.rs: FLUID_TIER → tier_N cfg feature flag emission
- rendering/src/device.rs: GpuContext headless async init, wgpu 29.x API [NEEDS_REVIEW: claude → BUG-008]
- rendering/src/surface.rs: RenderSurface swapchain [NEEDS_REVIEW: claude → BUG-009]
- rendering/src/tier0/mod.rs + renderer.rs: CpuFramebuffer + SoftbufferRenderer, GPU-free test frame, JPEG encode
- rendering/src/http_preview.rs: tiny_http server port 8080, SharedFrame Arc<Mutex>, /frame.jpg endpoint
- rendering/src/scene_renderer.rs: SceneRenderer trait (generic W: World), StubRenderer
- rendering/src/debug_overlay.rs: FrameStats, display string, CPU framebuffer banner stub
- rendering/src/camera.rs: Camera with core::units::Meters position, glam Mat4 view/projection
- rendering/src/pipeline/: module scaffold
- config/rendering.toml: preview_http_port=8080, frame resolution, jpeg_quality, camera defaults
cargo test -p rendering: 12 passed, 0 failed, EXIT:0
cargo check --workspace: EXIT:0
BUG-008 and BUG-009 filed in BUG_POOL.md (Pending Claude Review for C7)

C3 domain closed. File new rendering/ bugs to BUG_POOL.md, assign to C7 for triage.

---

[C4_INTERFACES_PUBLISHED]
Published by: C4 (session: c4_physics_core_20260429T230621Z)
Timestamp: 2026-04-29T23:06:21+05:30
Gate files verified:
- physics_core/src/integrators/traits.rs — Integrator + DerivativeProvider traits, Send+Sync, SI Seconds
- physics_core/src/collision/traits.rs — ConvexShape, ShapeRef, ContactManifold (depth: Meters), CollisionDetector, Broadphase
- physics_core/src/constraints/traits.rs — Constraint + ConstraintSolver traits, sequential impulse contract
- physics_core/Cargo.toml — tier features declared, glam workspace = true (0.32.1 verified), core path dep wired
- physics_core/build.rs — FLUID_TIER → tier_N cfg flag emission, additive tier inheritance
- config/physics_core.toml — constraint_solver_iterations=10, broadphase_cell_size=1.0
cargo test -p physics_core: 6 passed, 0 failed, EXIT:0
cargo check --workspace: EXIT:0

C5 (Sim Components) may now begin.

---

## Unresolved Items

| Tag | Location | Description | Owner |
|-----|----------|-------------|-------|
| `[RESOLVED: infeasible — no Rust SYCL bindings; wgpu covers Intel at Tiers 0-2]` | `knowledge/capability_tiers.md` | oneAPI (Intel) Tier 3 compute support | C5 |

---

## In-Progress Sessions

<!-- Format: [IN_PROGRESS: <agent_id> at <timestamp> on <task>] -->
<!-- Remove when session retires. C7 audits for stale entries. -->



---

## Retired Sessions

<!-- Format: [RETIRED: <agent_id> at <timestamp>] -->
<!-- Permanent record. Never delete. -->

[RETIRED: c1_core_20260428T022400Z at 2026-04-28T02:24:00+05:30]
[RETIRED: c3_rendering_20260428T173700Z at 2026-04-29T03:05:00+05:30]
[RETIRED: c4_physics_core_20260429T230621Z at 2026-04-29T23:06:21+05:30]
[RETIRED: c5_sim_components_20260429T214423Z at 2026-04-30T06:48:00+05:30]

---

## Reactivated Sessions

<!-- Format: [REACTIVATED: <agent_id> at <timestamp> for BUG-<id>] -->

---

## Incremental Commit Log

<!-- Format: <timestamp> <agent_id> <file count> <line count> <description> -->
<!-- Flag any commit touching >400 lines across >5 files without coordinator sign-off -->

---

## Notes

- `config/builder_flags.toml` must be created by C2 before any component coordinator adds flags.
- C5 oneAPI `[UNRESOLVED]` must be resolved before `[C5_COMPLETE]` is published.
- C7 must confirm all `[RETIRED]` entries have a corresponding `handoff_prompt.md` in `pack/`.
- knowledge/ files must not be written without incrementing the `<!-- version: N -->` counter.

[C4_COMPLETE]
Published by: C4 (session: c4_physics_core_continuation_20260429T164538Z)
Timestamp: 2026-04-29T16:45:38+05:30
All C4 post-gate work verified:
- physics_core/src/integrators/velocity_verlet.rs: implemented, energy conservation test passes
- physics_core/src/integrators/leap_frog.rs: implemented, consumed by C5
- physics_core/src/collision/gjk.rs: implemented, tested
- physics_core/src/collision/epa.rs: implemented, penetration depth accurate
- physics_core/src/constraints/sequential_impulse.rs: implemented, tested
- physics_core/Cargo.toml: faer 0.24 verified, [UNVERIFIED] removed
- config/physics_core.toml: dynamically consumed
cargo test -p physics_core: 22 passed, 0 failed, EXIT:0

C5 (Sim Components) may now proceed fully. C4 domain closed. File new physics_core/ bugs to BUG_POOL.md, assign to C7 for triage.

[ROOT_COMPLETE]

---

[C5_COMPLETE]
Published by: C5 (session: c5_sim_components_20260429T214423Z)
Timestamp: 2026-04-30T06:48:00+05:30
All C5 gate criteria verified:
- components/motion_force_simulator/src/lib.rs: gravity, spring-damper, hydraulic actuator, electric motor, joints; 7 tests pass
- components/fluid_simulator/src/sph.rs: Wendland C2 kernel (σ=21/16π, support=2h), XSPH, Tait EOS, Leap-Frog; kernel normalization test + 1000-step stability test pass
- components/fluid_simulator/src/cfd.rs: MAC grid, Chorin projection (Jacobi), 3 tests pass
- components/fluid_simulator/src/compute.rs: GpuComputeBackend trait + CUDA/ROCm stub backends [NEEDS_REVIEW: claude]
- components/aerodynamic_simulator/src/lib.rs: thin-aerofoil C_L/C_D model, ISA constants; 4 tests pass
- components/thermodynamic_simulator/src/lib.rs: lumped-capacitance + RK4; analytical accuracy test < 0.01 K error; 2 tests pass
- components/fem_structural/src/lib.rs: Euler-Bernoulli beam stiffness/mass, Newmark-Beta solver, faer LU; cantilever 1% accuracy gate test PASSES; 3 tests pass
- Solver selection: faer 0.24 over nalgebra-sparse — unified sparse+dense API, BLAS performance; documented in lib.rs
- oneAPI [UNRESOLVED] → [RESOLVED: infeasible]; see knowledge/capability_tiers.md version 2
- physics_core/src/integrators/newmark_beta.rs: implemented [NEEDS_REVIEW: claude]
- physics_core/src/integrators/rk4.rs: implemented [NEEDS_REVIEW: claude]
cargo check --workspace: EXIT:0 (zero errors, zero warnings)
cargo test (all C5 components): 26 passed, 0 failed

Notes:
- All Tier 3 FFI backends (CUDA/ROCm) are stubs tagged [NEEDS_REVIEW: claude] — production FFI wiring is a Tier 2+ task
- SPH kernel constant corrected: Dehnen & Aly (2012) reports σ=21/(2π) for support=h; this code uses support=2h → σ=21/(16π)
- C7 review queue: newmark_beta.rs, rk4.rs, sph.rs, cfd.rs, compute.rs (fluid_simulator), fem_structural/src/lib.rs

C5 domain closed. File new components/ bugs to BUG_POOL.md, assign to C7 for triage.
C6 (Debugger) may now begin. C7 (Quality Gate) review queue populated.
