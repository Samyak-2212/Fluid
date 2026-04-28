# fluid_simulator

Planned fluid simulation component crate for SPH and CFD workflows.

## What It Does

`fluid_simulator` is intended to hold the Fluid workspace's particle and grid-based fluid simulation component. The coordinator specification assigns SPH and CFD responsibilities to this crate, and the crate already declares dependencies on `core` and `physics_core`.

The current source state is minimal. `components/fluid_simulator/src/lib.rs` contains only a stub comment, and `knowledge/project_manifest.md` shows the simulation-components coordinator as blocked behind C4. This crate therefore has manifest structure but no verified public simulation API yet.

Because the implementation gate has not been published, any behavior beyond the manifest and coordinator references should be treated as [UNVERIFIED - coordinator gate not yet published].

## Capability Tier

| Mode | Minimum tier | Status |
|---|---|---|
| SPH low-resolution fallback | 0 | [UNVERIFIED - coordinator gate not yet published] |
| Medium SPH | 1 | [UNVERIFIED - coordinator gate not yet published] |
| High-resolution CFD / compressible flow | 2 | [UNVERIFIED - coordinator gate not yet published] |
| Tier 3 compute backends | 3 | [UNVERIFIED - coordinator gate not yet published] |

## Quick Start

```toml
[dependencies]
fluid_simulator = { path = "../components/fluid_simulator" }
```

```rust
use fluid_simulator as _;

fn main() {}
```

```bash
cargo build -p fluid_simulator
```

## Build Instructions

```bash
cargo build -p fluid_simulator
FLUID_TIER=0 cargo build -p fluid_simulator
FLUID_TIER=2 cargo build -p fluid_simulator
```

The crate declares `tier_0` through `tier_3` features and depends on `core` and `physics_core`. No crate-local `build.rs` is present at the moment.

## Known Limitations

- No verified public API exists in `src/lib.rs`; the crate is currently a stub.
- `knowledge/project_manifest.md` shows C5 blocked pending C4 interfaces and completion work.
- No open `BUG_POOL.md` entry is currently tagged specifically to `components/fluid_simulator`.
