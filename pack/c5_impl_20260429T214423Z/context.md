# C5 Sim Components — Implementation Session Pack
# Session: c5_sim_components_20260429T214423Z
# Written at tool-call boundary (rule: pack after 15 calls, then continue)

## Status at Checkpoint

### Completed
- oneAPI `[UNRESOLVED]` tag **resolved** as `[RESOLVED: infeasible]`.
  - Rationale documented in `knowledge/capability_tiers.md` (version 2).
  - Manifest updated in `knowledge/project_manifest.md` (version 11).
- `physics_core/src/integrators/newmark_beta.rs` — full implicit Newmark-Beta
  (γ=0.5, β=0.25) implementation with 2 tests. [NEEDS_REVIEW: claude]
- `physics_core/src/integrators/rk4.rs` — full RK4 implementation with
  `Rk4State` trait, `Rk4DerivativeProvider` trait, `ScalarState`, and 2 tests.
  [NEEDS_REVIEW: claude]
- C5 status updated to IN_PROGRESS in project_manifest.md.

### In Progress (remaining work this session)
- `components/motion_force_simulator/` — force application, actuators, joints
- `components/fluid_simulator/` — SPH (Wendland C2 + XSPH + LeapFrog) + CFD stub
- `components/aerodynamic_simulator/` — lift/drag/thrust force model
- `components/thermodynamic_simulator/` — operator splitting + RK4
- `components/fem_structural/` — linear FEM + Newmark-Beta solver
- Compute FFI traits (CUDA/ROCm) in each Tier 3 component
- `cargo check --workspace` verification

## Key Findings / Constraints

- `physics_core::rigid_body::RigidBody` has `inv_mass() -> f64` but multiplies
  by `f32` Vec3 in `apply_impulse`. This is a C4 impl detail; C5 must not modify
  rigid_body — consume the public API only.
- `physics_core::integrators::newmark_beta` is gated on `#[cfg(feature = "tier_1")]`.
  `fem_structural` must enable `tier_1` feature of `physics_core` in its Cargo.toml.
- `physics_core::integrators::rk4` same: `tier_1` gate.
- All C5 components must re-export `GpuComputeBackend` trait behind `tier_3` feature.
- `glam` needed as workspace dep in all component Cargo.toml files.

## Files Touched
- `knowledge/project_manifest.md` (version 11)
- `knowledge/capability_tiers.md` (version 2)
- `physics_core/src/integrators/newmark_beta.rs` (implemented)
- `physics_core/src/integrators/rk4.rs` (implemented)

## Next Steps (resume from here)
1. Add `glam = { workspace = true }` to all 5 component Cargo.toml files.
2. Implement `motion_force_simulator/src/lib.rs` (force, actuators, joints).
3. Implement `fluid_simulator/src/` (SPH + CFD).
4. Implement `aerodynamic_simulator/src/lib.rs`.
5. Implement `thermodynamic_simulator/src/lib.rs`.
6. Implement `fem_structural/src/` (assembler + Newmark-Beta solver).
7. Add compute FFI traits (CUDA/ROCm) per component in `src/compute/`.
8. Run `cargo check --workspace`.
9. Run `cargo test --workspace` (must pass C5 completion gate tests).
10. Write `[C5_COMPLETE]` signal → **hard retirement trigger**.
