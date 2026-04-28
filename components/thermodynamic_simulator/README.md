# thermodynamic_simulator

Early thermodynamics component crate with a single initialization entry point.

## What It Does

`thermodynamic_simulator` is reserved for thermal simulation within the Fluid workspace. The coordinator prompt assigns operator-splitting and RK4-based thermodynamic updates to this crate, with configuration expected under `config/thermodynamic_simulator.toml`.

The verified source is still very small: `src/lib.rs` exports only `pub fn init()` and is marked `[NEEDS_REVIEW: claude]`. The project manifest also shows the simulation-components domain as blocked, so detailed thermal behavior is not yet available in source.

This crate should therefore be treated as a scaffold with a smoke-test entry point rather than a finished thermodynamics API.

## Capability Tier

| Feature area | Minimum tier | Status |
|---|---|---|
| Thermodynamic component integration | 1 | [UNVERIFIED - coordinator gate not yet published] |
| Higher-fidelity coupled thermal models | 2 | [UNVERIFIED - coordinator gate not yet published] |

## Quick Start

```toml
[dependencies]
thermodynamic_simulator = { path = "../components/thermodynamic_simulator" }
```

```rust
use thermodynamic_simulator::init;

fn main() {
    init();
}
```

```bash
cargo build -p thermodynamic_simulator
```

## Build Instructions

```bash
cargo build -p thermodynamic_simulator
FLUID_TIER=1 cargo build -p thermodynamic_simulator
```

The crate declares `tier_0` through `tier_3` and depends on `core` plus `physics_core`.

## Known Limitations

- The only verified public API is `pub fn init()`.
- The source file is tagged `[NEEDS_REVIEW: claude]`.
- Runtime configuration for thermal properties has not been added yet.
