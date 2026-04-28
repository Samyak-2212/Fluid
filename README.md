# Fluid

Rust workspace for the Fluid simulation framework.

## What It Does

This repository defines a multi-crate workspace for simulation, rendering, build orchestration, and debugging. The workspace root is not a library crate; it is the assembly point for `core`, `physics_core`, `rendering`, `builder`, `debugger`, and the simulation component crates under `components/`.

The workspace encodes common policies that every member crate inherits: Rust 2021 edition, shared dependency versions for `glam` and `rayon`, and compile-time capability-tier selection through `FLUID_TIER`. Component crates are opt-in and are selected through Cargo features or the builder UI.

Coordinator status matters to anyone using this repository. `core/` and `builder/` have published completion gates, while `rendering/`, `physics_core/`, `debugger/`, and the simulation components are still partially blocked in `knowledge/project_manifest.md`. The docs in those areas therefore distinguish verified source from planned coordinator behavior.

## Capability Tier

| Tier | Hardware profile | Workspace expectation |
|---|---|---|
| 0 | CPU only, no GPU | `core`, `physics_core`, and Tier 0 component fallbacks must compile without GPU assumptions. |
| 1 | Integrated GPU | Enables medium-fidelity simulation and `wgpu` rendering paths. |
| 2 | Discrete GPU | Targets higher-fidelity rendering and physics workloads. |
| 3 | Multi-GPU / HPC | Reserved for coupled multi-physics and CUDA/ROCm FFI owned by C5. |

Tier selection is compile-time only. The root workspace does not switch tiers at runtime.

## Quick Start

Add the workspace dependency you want to consume rather than the root itself:

```toml
[dependencies]
core = { path = "core" }
```

Minimal example against the currently completed `core` crate:

```rust
use core::ecs::{ArchetypeWorld, World};

fn main() {
    let mut world = ArchetypeWorld::new();
    let entity = world.spawn();
    world.insert(entity, 123u32);
    assert_eq!(world.get::<u32>(entity), Some(&123));
}
```

Build the workspace root:

```bash
cargo build
```

## Build Instructions

Use the repository-standard commands from `AGENTS.md`:

```bash
cargo build
FLUID_TIER=0 cargo build
FLUID_TIER=2 cargo build --features rendering
```

Relevant feature and environment controls at the workspace level:

- `FLUID_TIER=0|1|2|3` selects compile-time capability tier.
- Component crates are opt-in from their own manifests, for example `--features fluid_simulator`.
- Shared dependency versions are declared in the workspace root and reused by member crates.

## Known Limitations

- `BUG-001`: `core/ecs` is not dyn-compatible as `dyn World`, and the workspace bug pool records this as an open issue affecting root-level `cargo build` expectations.
- `rendering/`, `physics_core/`, `debugger/`, and the component crates have not all published completion gates, so some workspace-level paths remain partial or stubbed.
