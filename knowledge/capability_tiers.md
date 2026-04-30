<!-- version: 2 -->
# Capability Tiers

Every feature in the Fluid framework must declare its minimum tier.
No coordinator or implementation agent may target "all tiers equally."
Tier 0 must always have a working fallback. Quality does not degrade — scope reduces per tier.

## Tier Table

| Tier | Hardware Profile | Physics Mode | Render Backend | Accuracy Target |
|------|-----------------|--------------|----------------|-----------------|
| 0 | CPU only, ≤2GB RAM, no GPU | Simplified: rigid body, basic fluid (SPH low-res) | CPU software rasterizer (softbuffer crate) | Real-time interactive, reduced precision |
| 1 | Integrated GPU (Intel HD/Arc, AMD APU) | Full rigid + soft body, medium SPH, basic FEM | wgpu with OpenGL backend | Interactive scientific |
| 2 | Discrete GPU (Nvidia/AMD) | Full FEM, high-res CFD, compressible flow | wgpu (Vulkan/DX12/Metal) | Scientific publication accuracy |
| 3 | Multi-GPU / HPC node | Full coupled multi-physics | wgpu compute + CUDA/ROCm via FFI | Aerospace / structural engineering grade |

## Graphics and Compute API Strategy

`wgpu` is the sole graphics and general compute abstraction layer for Tiers 0–2.
Do NOT implement separate Vulkan, DX12, Metal, or OpenGL backends manually.
`wgpu` selects the best available backend at runtime automatically:

- Vulkan on Linux, Windows, Android
- DX12 on Windows
- Metal on macOS, iOS
- OpenGL / OpenGL ES via ANGLE as fallback
- WebGPU in browser (IDX preview)

For Tier 3 only, CUDA (Nvidia) and ROCm/HIP (AMD) compute are supported via direct FFI bridges.
These are owned exclusively by the C5 Simulation Components Coordinator and must be isolated behind
a trait interface. No other crate in the workspace may have a direct dependency on CUDA or ROCm.

oneAPI (Intel) support is `[RESOLVED: infeasible — no mature Rust SYCL bindings exist]`.

Rationale (assessed by C5, session c5_sim_components_20260429T214423Z):
- SYCL is a C++ single-source programming model; kernels and host code are compiled together.
- Rust `extern "C"` FFI cannot directly call template-heavy C++ SYCL APIs.
- `bindgen` cannot handle the template-heavy C++ headers used by the SYCL runtime.
- No production-ready crate on crates.io provides SYCL bindings as of 2026-04.
- Intel GPUs (Arc, HD) are fully covered by `wgpu` at Tiers 0–2 via Vulkan and OpenGL backends.
- If Intel-specific compute is re-evaluated in future, `opencl-sys` (OpenCL C API, crates.io)
  is the viable path — OpenCL is exposed by every Intel oneAPI runtime installation.

Fallback: Intel hardware at Tier 3 uses the `wgpu` compute backend (Tier 2 ceiling for Intel).
This is acceptable; Tier 3 HPC targets are CUDA (Nvidia) and ROCm/HIP (AMD) only.

Any deviation from `wgpu` for Tiers 0–2 must be justified in `knowledge/project_manifest.md`.

## Tier Selection at Build Time

Tier is set via the `FLUID_TIER` environment variable (values: `0`, `1`, `2`, `3`).
Default is `0` for debug builds, `2` for release builds.

`build.rs` in each crate reads `FLUID_TIER` and emits the corresponding Cargo feature flag:
`cargo:rustc-cfg=feature="tier_N"` (e.g. `tier_0`, `tier_1`).

This enables `#[cfg(feature = "tier_N")]` gating throughout the codebase.
Each crate's `Cargo.toml` must declare `tier_0`, `tier_1`, `tier_2`, `tier_3` as explicit features.

> **Important:** Tier selection is compile-time only — it is baked into the binary at build time via
> Cargo features. There is no runtime tier switching. Each tier produces a separate binary.
> The builder UI tier selector sets `FLUID_TIER` before invoking `cargo`; changing the tier
> requires a full recompile. Do not attempt to implement runtime tier detection or dynamic dispatch
> based on tier — use `#[cfg(feature = "tier_N")]` exclusively.

## Tier Feature Flag Declaration (per crate Cargo.toml)

```toml
[features]
default = []
tier_0 = []
tier_1 = []
tier_2 = []
tier_3 = []
```

## Euler Integration Gate

Euler integration is only permitted in Tier 0 simplified mode.
It must be gated with the `tier_0` Cargo feature flag:

```rust
#[cfg(feature = "tier_0")]
// Euler integrator — Tier 0 only
```

No Euler integration in any scientific accuracy path (Tiers 1–3).
