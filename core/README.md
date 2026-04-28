# core

Foundational interfaces and utility types for the Fluid framework.

## What It Does

The `core` crate provides the shared building blocks that the rest of the workspace depends on: SI unit newtypes, ECS traits and a concrete `ArchetypeWorld`, an event-bus abstraction plus a local implementation, math re-exports, a fixed-timestep manager, and a thread-pool abstraction backed by Rayon.

This is the most complete crate in the repository as of the current manifest state. `knowledge/project_manifest.md` records both `[C1_INTERFACES_PUBLISHED]` and `[C1_COMPLETE]`, so the APIs documented here are backed by shipped source rather than coordinator-only plans.

Downstream crates use `core` to avoid duplicating units, world interfaces, and shared math/threading conventions. `physics_core` consumes `core::units` and ECS traits directly, while `rendering` depends on `core::math`, `core::units`, and `core::ecs`.

## Capability Tier

| Feature area | Minimum tier | Notes |
|---|---|---|
| ECS traits and `ArchetypeWorld` | 0 | CPU-safe baseline used by all higher tiers. |
| SI unit newtypes | 0 | Required across all physics-facing boundaries. |
| Fixed timestep manager | 0 | Tier-independent; config-driven. |
| Rayon-backed thread pool | 0 | Present at all tiers, with more benefit at Tier 1+. |

## Quick Start

```toml
[dependencies]
core = { path = "../core" }
```

```rust
use core::ecs::{ArchetypeWorld, World};
use core::units::Seconds;

fn main() {
    let mut world = ArchetypeWorld::new();
    let entity = world.spawn();
    world.insert(entity, Seconds(0.016));
    assert_eq!(world.entity_count(), 1);
}
```

```bash
cargo build -p core
```

## Build Instructions

```bash
cargo build -p core
FLUID_TIER=0 cargo build -p core
FLUID_TIER=2 cargo build -p core
```

The crate declares explicit `tier_0` through `tier_3` features. `core/build.rs` reads `FLUID_TIER` and emits the matching `cfg(feature = "tier_N")` flag.

## Known Limitations

- `BUG-001`: the `World` trait is not dyn-compatible because several methods are generic, so APIs must use type parameters rather than `dyn World`.
- `memory` and `scene` are still stub modules and do not yet expose production-ready abstractions.
- `event_bus_impl.rs`, `time/mod.rs`, and `ecs/world.rs` carry `[NEEDS_REVIEW: claude]` markers in source.
