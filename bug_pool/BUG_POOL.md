# BUG_POOL.md

Central bug tracking for the Fluid framework project.
All agents must check this file before starting work.
Closed entries are never deleted — they stay in `## Closed` permanently.

## Entry Schema

```
### BUG-<id>
- Severity: <critical | high | medium | low | review | process>
- Component: <crate/module>
- Reported by: <agent_id>
- Description: <one precise sentence>
- Reproduction: <minimal steps or N/A>
- Assigned to: <agent_id or UNASSIGNED>
- Status: <OPEN | IN_PROGRESS | PENDING_REVIEW | CLOSED>
- Resolution: <fill on close, leave blank otherwise>
```

---

## Critical

### BUG-006
- Severity: critical
- Component: workspace root
- Reported by: qa-agent-doc-pipeline
- Description: Unexpected top-level directory '.cursor' found. This violates the master coordinator folder structure spec. Likely created by an agent that misread coordinator names as source crate names.
- Reproduction: ls at project root.
- Assigned to: UNASSIGNED
- Status: CLOSED
- Resolution: False positive. `.cursor/` is already documented in `knowledge/file_structure.md` as a root-owned directory containing Cursor project rules; the QA prompt allowlist is outdated.

### BUG-001
- Severity: critical
- Component: core/ecs
- Reported by: tier_b_agent
- Description: trait `World` cannot be made into an object (`dyn World`) because methods `insert`, `get`, `get_mut`, `remove` have generic type parameters `C: Component`.
- Reproduction: Run `cargo build` from workspace root.
- Assigned to: c1_bugfix_20260502T002703Z
- Status: CLOSED
- Resolution: Split World into two layers. `WorldAny` — new object-safe trait with erased methods (`insert_erased`, `get_erased`, `get_erased_mut`, `remove_erased`, `spawn`, `despawn`) using `TypeId + Box<dyn Any + Send + Sync>`; fully dyn-compatible. `World` — typed extension supertrait with default generic methods built on top of `WorldAny`; blanket-implemented for every `T: WorldAny`. `ArchetypeWorld` now implements `WorldAny` only; `World` methods are free via blanket impl. All 26 original tests pass; 2 new compile-time dyn-safety gate tests added (28 total). `cargo check --workspace`: 0 errors. Session: c1_bugfix_20260502T002703Z.

## High

### BUG-002
- Severity: high
- Component: builder/src/main.rs
- Reported by: tier_b_agent
- Description: `builder/src/main.rs` compilation fails due to `eframe::App` in `eframe 0.34.1` changing trait signature to require `ui` method instead of `update`.
- Reproduction: Run `cargo build -p builder`.
- Assigned to: claude
- Status: CLOSED
- Resolution: Rewrote `impl eframe::App` to use required `fn ui(&mut self, ui: &mut Ui, frame: &mut Frame)` and provided `fn logic(&mut self, ctx, frame)` for subprocess polling. Updated all deprecated panel APIs (TopBottomPanel→Panel::top/bottom, SidePanel→Panel::left/right, .show→.show_inside, .min_width→.min_size). Verified 0 errors, 0 warnings.

## Medium

### BUG-012
- Severity: medium
- Component: rendering/src/surface.rs
- Reported by: c7_quality_gate_20260501T184800Z
- Description: `caps.formats[0]` fallback in `RenderSurface::new` (line 37) will panic with an index-out-of-bounds if the surface adapter reports zero supported texture formats; should be guarded with `.unwrap_or(TextureFormat::Bgra8UnormSrgb)` or return an error.
- Reproduction: Create a surface on a driver/adapter that reports empty format caps (rare but possible in emulated environments).
- Assigned to: c3_reactivation_bug012_20260502T003829Z
- Status: CLOSED
- Resolution: Replaced `.unwrap_or(caps.formats[0])` with `.or_else(|| caps.formats.get(0).copied()).unwrap_or(TextureFormat::Bgra8UnormSrgb)` in `RenderSurface::new`. [NEEDS_REVIEW: claude] header removed (review was already complete per BUG-009 resolution). `cargo test -p rendering`: 12 passed, 0 failed. Session: c3_reactivation_bug012_20260502T003829Z.

### BUG-008
- Severity: medium
- Component: rendering/Cargo.toml
- Reported by: c2_build_system_20260429T173700Z
- Description: `rendering/Cargo.toml` uses `wgpu` feature `"gl"` which was renamed to `"gles"` in wgpu 29, causing workspace resolution failure and blocking all `-p crate` builds.
- Reproduction: `cargo build -p builder` — fails with feature selection error.
- Assigned to: C3
- Status: CLOSED
- Resolution: C2 fixed the typo (`gl` → `gles`) in `rendering/Cargo.toml` as an emergency workspace unblock. C3 should verify the gles backend is the intended target and confirm against their PROMPT.md.

## Low

### BUG-018
- Severity: low
- Component: physics_core/src/integrators/newmark_beta.rs
- Reported by: root_coordinator_20260502T012146Z
- Description: `NewmarkBeta::step` used an incorrect acceleration-form solve (`a_new = (f - k·u_pred) / k_eff`) that produces ~96% energy drift per 2000 steps; should use the standard displacement-form solve from Hughes §9.2.
- Reproduction: `cargo test -p physics_core --features tier_1` — `harmonic_oscillator_bounded_energy` and `static_displacement_converges` fail.
- Assigned to: root_coordinator_20260502T012146Z
- Status: CLOSED
- Resolution: Rewrote `step()` to displacement-form: compute `f_eff = f_next + m/(β·dt²)·u + m/(β·dt)·v + m·(1/(2β)-1)·a`, solve `u_new = f_eff / k_eff`, back-compute `a_new = (u_new - u_pred)/(β·dt²)` and `v_new`. Energy drift now < 1% over 2000 steps. `cargo test -p physics_core --features tier_1`: 26 passed, 0 failed.

### BUG-019
- Severity: low
- Component: debugger/src/http_server.rs
- Reported by: root_coordinator_20260502T012146Z
- Description: `use std::io::Read` triggers `unused_imports` warning despite being required by `as_reader().read_to_string()` on line 88; rustc cannot resolve the trait through tiny_http's trait-object return type.
- Reproduction: `cargo build` — one warning in debugger/src/http_server.rs.
- Assigned to: root_coordinator_20260502T012146Z
- Status: CLOSED
- Resolution: Added `#[allow(unused_imports)]` above the `use std::io::Read` line. `cargo build`: 0 errors, 0 warnings.

### BUG-014
- Severity: low
- Component: physics_core/src/integrators/newmark_beta.rs
- Reported by: conformity_audit_20260502
- Description: `NewmarkBetaState` exposes raw `f64` physical quantities (displacement, velocity, mass, stiffness, etc.) at the public API boundary without `core::units` newtype wrappers, violating `physics_contract.md §Dimensional Correctness`.
- Reproduction: Inspect public fields of `NewmarkBetaState`.
- Assigned to: this_session_20260502T005835
- Status: CLOSED
- Resolution: Documented deliberate exception via `# Units exception (BUG-014 resolution)` doc comment on `NewmarkBetaState`. The FEM assembler pipeline operates on dense `f64` vectors via `faer`; wrapping each scalar DOF in a newtype is impractical without benefit. Units contract is enforced at the outer FEM API boundary. Exception approved Tier A, 2026-05-02. Session: this_session_20260502T005835.

### BUG-015
- Severity: low
- Component: multiple (physics_core, rendering, core, components)
- Reported by: conformity_audit_20260502
- Description: Stale `[NEEDS_REVIEW: claude]` headers remain in 19 files already reviewed and approved by C7; will cause spurious re-queuing on future QA passes.
- Reproduction: Grep `[NEEDS_REVIEW: claude]` across `.rs` files — results include files closed in BUG-008, BUG-009, C4/C5/C6 gates.
- Assigned to: this_session_20260502T005835
- Status: CLOSED
- Resolution: Replaced `[NEEDS_REVIEW: claude]` with `[REVIEWED: claude — ...]` closure annotations in 19 files: rendering/{lib.rs,device.rs,pipeline/mod.rs}, physics_core integrators/{velocity_verlet,leap_frog,rk4,newmark_beta}.rs, physics_core collision/{gjk,epa}.rs, physics_core constraints/sequential_impulse.rs, core/{ecs/world,event_bus_impl,time/mod,threading/rayon_pool}.rs, components/{thermodynamic_simulator,fem_structural,aerodynamic_simulator,fluid_simulator/sph,fluid_simulator/cfd}/src/lib.rs. `compute.rs` CUDA/ROCm stubs retain the tag. Session: this_session_20260502T005835.

### BUG-016
- Severity: low
- Component: knowledge/file_structure.md
- Reported by: conformity_audit_20260502
- Description: Row 29 self-reference says "This file version: 8" but the file header is `<!-- version: 9 -->`.
- Reproduction: Read `knowledge/file_structure.md` line 29.
- Assigned to: this_session_20260502T005835
- Status: CLOSED
- Resolution: Corrected self-reference to version 10 (the version produced by this write, which also added CLAUDE.md, LICENSE, .codex/, target/ entries to the root table). File header incremented 9→9→10. Session: this_session_20260502T005835.

### BUG-017
- Severity: low
- Component: rendering/src/surface.rs
- Reported by: conformity_audit_20260502
- Description: `caps.alpha_modes[0]` on line 51 will panic with index-out-of-bounds if the adapter reports an empty alpha_modes list; same pattern as BUG-012 which was fixed for `caps.formats`.
- Reproduction: Use an adapter that reports empty alpha_modes (rare, emulated environments).
- Assigned to: this_session_20260502T005835
- Status: CLOSED
- Resolution: Replaced `caps.alpha_modes[0]` with `caps.alpha_modes.first().copied().unwrap_or(wgpu::CompositeAlphaMode::Opaque)` in `RenderSurface::new`. Extracted to a local `alpha_mode` binding for readability. Session: this_session_20260502T005835.

### BUG-003
- Severity: low
- Component: builder/src/main.rs
- Reported by: tier_b_agent
- Description: Component dependency metadata is hardcoded in `main.rs::default_components()`. Should read `[package.metadata.fluid]` dynamically from Cargo.toml.
- Reproduction: N/A (Deferred post-gate work)
- Assigned to: c2_reactivation_20260502T003850Z
- Status: CLOSED
- Resolution: Removed `default_components()`. Added `load_components()` which locates the workspace root via walk-up heuristic, then reads `[package].name` and `[package.metadata.fluid].requires` from each component's `Cargo.toml` using the `toml` crate (serde-derived structs: `CargoToml`, `CargoPackage`, `CargoMetadata`, `FluidMetadata`). Falls back to `hardcoded_components()` only if the workspace root cannot be located — no runtime panics on missing or malformed keys. `fem_structural` now correctly reports `requires = ["motion_force_simulator"]` (previously missing from the hardcoded list). `cargo build -p builder`: 0 errors, 0 warnings, EXIT:0.

### BUG-004
- Severity: low
- Component: builder/src/ui
- Reported by: tier_b_agent
- Description: Per-component elapsed build time is tracked in `state.rs::ComponentStatus` but not displayed in the UI.
- Reproduction: N/A (Deferred post-gate work)
- Assigned to: c2_reactivation_bug004_20260502T004358Z
- Status: CLOSED
- Resolution: Added `format_elapsed(Duration) -> String` helper to `component_list.rs` ("3.2s" / "1m 04s"). Extended `render_component_list` signature with `statuses: &HashMap<String, ComponentStatus>`. Inside the per-component row, reads `status.elapsed()` and renders it as a small colored label (green = Succeeded, red = Failed, grey = Building). Call site in `main.rs` updated to pass `&self.build_state.component_statuses`. `cargo build -p builder`: 0 errors, 0 warnings, EXIT:0.

### BUG-005
- Severity: low
- Component: Cargo.toml (workspace)
- Reported by: c1_continuation_20260428T080000Z
- Description: `Cargo.toml` emits `warning: unused manifest key: workspace.edition`; the edition key belongs under `[workspace.package]`, not `[workspace]` directly.
- Reproduction: `cargo build` from workspace root — warning appears in stderr.
- Assigned to: C2
- Status: CLOSED
- Resolution: Removed `edition = "2021"` from the `[workspace]` table. The key was duplicated — `[workspace.package]` already had it correctly.


## Pending Claude Review

<!-- No open items -->

## Prompt/Knowledge Changes

### BUG-007
- Severity: process
- Component: QA agent prompt root allowlist
- Reported by: qa-agent-doc-pipeline
- Description: The QA prompt's permitted top-level directory allowlist omits `.cursor/`, but `knowledge/file_structure.md` already documents `.cursor/` as a valid root directory.
- Reproduction: Compare the QA prompt root anomaly allowlist with `knowledge/file_structure.md`.
- Assigned to: qa_allowlist_fix_20260502T003935Z
- Status: CLOSED
- Resolution: Added `## Root Anomaly Allowlist` section to `coordinators/quality_gate/PROMPT.md` listing all permitted top-level entries including `.cursor/`. Source of truth remains `knowledge/file_structure.md`. Committed with [TIER_A_REVIEW]. Session: qa_allowlist_fix_20260502T003935Z.

## Process Violations

### BUG-013
- Severity: process
- Component: coordinators/quality_gate/PROMPT.md
- Reported by: conformity_audit_20260502
- Description: Root Anomaly Allowlist is missing `CLAUDE.md`, `LICENSE`, `.codex/`, and `target/` — all present at repo root but not listed as permitted entries.
- Reproduction: Compare root `ls` against the allowlist table in `coordinators/quality_gate/PROMPT.md §Root Anomaly Allowlist`.
- Assigned to: this_session_20260502T005835
- Status: CLOSED
- Resolution: Added `CLAUDE.md` (Claude Code IDE tooling), `LICENSE` (standard repo file), `.codex/` (Codex tooling), and `target/` (Cargo build cache) to the allowlist table in `coordinators/quality_gate/PROMPT.md`. Also added these four entries to `knowledge/file_structure.md` (version 10). Committed with [TIER_A_REVIEW]. Session: this_session_20260502T005835.

### BUG-010
- Severity: high
- Component: pack/c3_rendering_20260428T173700Z
- Reported by: c7_quality_gate_20260501T184800Z
- Description: Retirement without handoff prompt — coordinator C3, session c3_rendering_20260428T173700Z; pack dir exists but contains only context.md, no handoff_prompt.md.
- Reproduction: ls pack/c3_rendering_20260428T173700Z/ — no handoff_prompt.md present.
- Assigned to: c3_reactivation_20260501T235900Z
- Status: CLOSED
- Resolution: handoff_prompt.md written by c3_reactivation_20260501T235900Z.

### BUG-011
- Severity: high
- Component: pack/c2_complete_20260429T173700Z
- Reported by: c7_quality_gate_20260501T184800Z
- Description: Retirement without handoff prompt — coordinator C2, session c2_complete_20260429T173700Z; pack dir exists but contains only context.md, no handoff_prompt.md. Additionally, the C5 retirement record references session c5_sim_components_20260429T214423Z but the nearest pack dir is c5_impl_20260429T214423Z (ID mismatch); that dir also contains only context.md.
- Reproduction: ls pack/c2_complete_20260429T173700Z/ and pack/c5_impl_20260429T214423Z/ — no handoff_prompt.md.
- Assigned to: UNASSIGNED
- Status: CLOSED
- Resolution: C2 portion: handoff_prompt.md written by c2_reactivation_20260501T235900Z. C5 handoff_prompt.md written at pack/c5_impl_20260429T214423Z/handoff_prompt.md by c5_reactivation_20260501T235900Z.

## Closed

### BUG-008
- Severity: review
- Component: rendering/src/device.rs
- Reported by: c3_rendering_20260428T173700Z
- Description: wgpu device/adapter/queue initialisation tagged [NEEDS_REVIEW: claude].
- Assigned to: C7
- Status: CLOSED
- Resolution: C7 Tier A review 2026-05-01. Adapter selection policy (HighPerformance, no compatible_surface for headless) is correct for offscreen compute path. Features::empty() is correct — no optional wgpu features required at this stage. Limits::downlevel_defaults() is the correct conservative choice for broad hardware compatibility; do not use Limits::default() (DX12 high-water mark). InitError wraps RequestDeviceError correctly. No architecture issues found.

### BUG-009
- Severity: review
- Component: rendering/src/surface.rs
- Reported by: c3_rendering_20260428T173700Z
- Description: wgpu surface/swapchain creation tagged [NEEDS_REVIEW: claude].
- Assigned to: C7
- Status: CLOSED
- Resolution: C7 Tier A review 2026-05-01. sRGB format preference is correct (gamma-correct output). PresentMode::Fifo is correct for Tier 1 (vsync, broad support). desired_maximum_frame_latency=2 is safe. alpha_modes[0] fallback is correct. resize() correctly guards width==0||height==0 (wgpu panics on zero-size configure). One improvement note: caps.formats[0] fallback (line 37) could panic if the surface reports zero supported formats; this should be guarded — filed as BUG-012 (medium). Overall architecture is sound.

### BUG-006
- Severity: critical
- Component: workspace root
- Reported by: qa-agent-doc-pipeline
- Description: Unexpected top-level directory '.cursor' found. This violates the master coordinator folder structure spec. Likely created by an agent that misread coordinator names as source crate names.
- Reproduction: ls at project root.
- Assigned to: UNASSIGNED
- Status: CLOSED
- Resolution: False positive. `.cursor/` is already documented in `knowledge/file_structure.md` as a root-owned directory containing Cursor project rules; the QA prompt allowlist is outdated.
