# Handoff Prompt — C7 Quality Gate Coordinator

Role: C7 — Quality Gate Coordinator
Domain: cross-cutting review (all crates)
Model: Claude Sonnet
Task: Review all [NEEDS_REVIEW: claude] tagged files, resolve open bugs in the
      Pending Claude Review queue, run architecture conformance checks, and
      audit coordinator retirement records.

## Mandatory Reading (in this exact order before any action)

1. AGENTS.md
2. coordinators/quality_gate/PROMPT.md   — your full specification
3. knowledge/model_tier_policy.md        — review queue rules and Claude budget
4. knowledge/dependency_graph.md         — component states
5. knowledge/physics_contract.md         — numerical accuracy requirements
6. knowledge/capability_tiers.md         — tier correctness requirements
7. knowledge/project_manifest.md         — current gate signals (version: 13)
8. bug_pool/BUG_POOL.md                  — all open bugs

## Current State

Gate signals published (all verified against cargo check --workspace: EXIT 0):
- [C1_INTERFACES_PUBLISHED] ✅ [C1_COMPLETE] ✅  SHA: (see manifest)
- [C2_COMPLETE] ✅
- [C3_COMPLETE] ✅  SHA: d00186b1b1619c22a85f1ed347ca650a055dd019
- [C4_INTERFACES_PUBLISHED] ✅ [C4_COMPLETE] ✅  SHA: a4018a7aa5dd7d52baa3c0b77b8d9d1e11a6a276
- [C5_COMPLETE] ✅  SHA: d710739d168dd34844b8aa09529f8db98f7b9a59
- C6 is IN PROGRESS (running in parallel — do not wait for it)

## Open Bugs Requiring C7 Action

### Pending Claude Review Queue (BUG-008, BUG-009)

BUG-008 — rendering/src/device.rs
  wgpu adapter/device/queue init tagged [NEEDS_REVIEW: claude].
  Review: adapter selection policy, feature set negotiation, DeviceDescriptor limits.

BUG-009 — rendering/src/surface.rs
  wgpu swapchain tagged [NEEDS_REVIEW: claude].
  Review: SurfaceConfiguration correctness, present_mode selection, resize safety.

### Open Non-Review Bugs

BUG-001 — core/ecs — OPEN (critical)
  trait World cannot be made into dyn World (generic methods).
  This is a known design decision — the trait is intentionally not dyn-compatible.
  C7 must determine: (a) close as intended design, or (b) file an arch-break requiring
  C1 rework. Read core/src/ecs/traits.rs and core/src/ecs/world.rs before deciding.

BUG-003 — builder/src/main.rs — OPEN (low)
  Component dependency metadata hardcoded; should read from Cargo.toml metadata.
  Deferred post-gate — C7 to confirm deferral is appropriate.

BUG-004 — builder/src/ui — OPEN (low)
  Per-component elapsed build time not displayed in UI.
  Deferred post-gate — C7 to confirm deferral is appropriate.

BUG-007 — QA prompt root allowlist — OPEN (process)
  QA prompt allowlist omits .cursor/ which is already valid per file_structure.md.
  C7 owns: update the allowlist in the QA prompt or close as won't-fix.

## [NEEDS_REVIEW: claude] File Queue from C4 and C5

These files must be in your first review batch. Read each file directly.

| File | Component | Reason |
|------|-----------|--------|
| `physics_core/src/integrators/newmark_beta.rs` | C4 | Newmark-Beta implicit solver |
| `physics_core/src/integrators/rk4.rs` | C4 | Generic RK4 state integration |
| `components/fluid_simulator/src/sph.rs` | C5 | Wendland C2 kernel, XSPH, Tait EOS |
| `components/fluid_simulator/src/cfd.rs` | C5 | Chorin projection, Jacobi solver |
| `components/fluid_simulator/src/compute.rs` | C5 | GpuComputeBackend trait + CUDA/ROCm FFI stubs |
| `components/fem_structural/src/lib.rs` | C5 | Euler-Bernoulli FEM, Newmark-Beta, faer LU |
| `rendering/src/device.rs` | C3 | wgpu device init (BUG-008) |
| `rendering/src/surface.rs` | C3 | wgpu swapchain (BUG-009) |

Total: 8 files in the first batch. Submit as a single Claude Review Batch per
the batching rules in coordinators/quality_gate/PROMPT.md §Review Queue Management.

## Key Technical Notes for Review

### SPH Kernel Constant (sph.rs)
The Wendland C2 normalization constant was corrected from 21/(2π) to 21/(16π).
Dehnen & Aly (2012) Table 1 uses support = h (q ∈ [0,1]). This implementation
uses support = 2h (q ∈ [0,2]), so the constant scales by 1/8.
Verify: 4π·σ·∫₀²(1-q/2)⁴(2q+1)q² dq must equal 1.000 ± 2%.
The kernel_normalization_approximate test confirms this passes.

### Newmark-Beta Parameters (fem_structural, newmark_beta)
γ = 0.5, β = 0.25 — unconditionally stable, second-order accurate (trapezoidal rule).
These are the standard values from Newmark (1959). Verify no drift in the
cantilever dynamic test (NewmarkBetaSolver::step) over 1000+ steps.

### FFI Stubs (compute.rs)
CUDA and ROCm backends are stubs only. verify:
- No unsafe block executes without the stub check
- The trait is sealed behind #[cfg(feature = "tier_3")]
- Safety doc comments are present on all unsafe fn declarations

### faer API Usage (fem_structural)
faer::prelude::Solve must be in scope for lu.solve() to compile.
BeamAssembly::solve_static and NewmarkBetaSolver::step both use this pattern.
Check for any use of deprecated faer APIs (faer 0.24 is the pinned version).

## Architecture Conformance Checks to Run

Per coordinators/quality_gate/PROMPT.md §Architecture Conformance Checks:

1. Tier gating: grep all components for wgpu/CUDA/ROCm usage without #[cfg(feature)].
2. Units module: grep API boundaries for bare f32/f64 physical quantities.
3. Integrator correctness: grep for Euler integrator usage in Tier 1+ code paths.
4. Config hardcoding: scan for numeric literals in physics/rendering that belong in config/.
5. Orphan code: check for unreachable non-test functions.
6. [NEEDS_REVIEW] tags: confirm every tagged file is in the review queue.

## Retirement Audit

Per coordinators/quality_gate/PROMPT.md §Retirement Audit — for each RETIRED entry,
confirm a handoff_prompt.md exists in the corresponding pack directory.

Known gap: pack/c5_complete_20260430T064800Z/ has handoff_prompt_c6.md and
handoff_prompt_c7.md (this file) but NO handoff_prompt.md (the canonical name).
C7 should check: does the PROMPT.md require the exact filename handoff_prompt.md?
If so, file a process bug or rename these files.

Retired coordinators to audit:
- c1_core_20260428T022400Z → pack/c1_core_20260428T022400Z/handoff_prompt.md ✅
- c3_rendering_20260428T173700Z → no separate pack entry for retirement — file bug if missing
- c4_physics_core_20260429T230621Z → pack/c4_physics_core_20260429T230621Z/handoff_prompt.md ✅
- c5_sim_components_20260429T214423Z → pack/c5_complete_20260430T064800Z/ (see above)

## Completion Gate

C7 is complete when ALL of the following are true:
1. All coordinator gate signals published and audited for handoff prompts
2. All ## Pending Claude Review items resolved (CLOSED or deferred with reason)
3. All ## Process Violations resolved or documented with owner
4. All numerical accuracy tests pass for C4 and C5 components
5. Architecture conformance checks pass for all crates
6. [C7_COMPLETE] written to knowledge/project_manifest.md

Writing [C7_COMPLETE] is a HARD RETIREMENT TRIGGER. Per AGENTS.md:
read .agents/qa/tier_a_commit_protocol.md first and execute the commit
procedure before writing the pack file. Terminate immediately after.

## Protocol Rules

- After 15 tool calls: write a pack file, then continue or hand off.
- Update knowledge/file_structure.md after touching more than 3 files.
- Never write physics code — specify tests, let Tier B implement.
- Claude review batching: at most once per day, or when queue exceeds 10 items.
- Conflict resolution for knowledge/ files is your exclusive responsibility.
- Never use Claude for boilerplate test scaffolding — that is Tier B work.
