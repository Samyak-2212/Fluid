# aerodynamic_simulator

Planned aerodynamic force simulation component crate.

## What It Does

`aerodynamic_simulator` is reserved for aerodynamic force modeling within the Fluid workspace. The coordinator specification assigns lift, drag, and thrust calculations to this crate, with `core` and `physics_core` as its upstream dependencies.

The verified source surface is currently limited to the manifest and a stub `src/lib.rs` comment. `knowledge/project_manifest.md` shows the simulation-components coordinator as blocked, so the crate should be treated as scaffolded rather than implemented.

Any references to air-density handling, body coupling, or force production beyond the crate manifest are [UNVERIFIED - coordinator gate not yet published].

## Capability Tier

| Feature area | Minimum tier | Status |
|---|---|---|
| Aerodynamic body coupling | 1 | [UNVERIFIED - coordinator gate not yet published] |
| Higher-fidelity aerodynamic models | 2 | [UNVERIFIED - coordinator gate not yet published] |
| Tier 3 GPU compute paths | 3 | [UNVERIFIED - coordinator gate not yet published] |

## Quick Start

```toml
[dependencies]
aerodynamic_simulator = { path = "../components/aerodynamic_simulator" }
```

```rust
use aerodynamic_simulator as _;

fn main() {}
```

```bash
cargo build -p aerodynamic_simulator
```

## Build Instructions

```bash
cargo build -p aerodynamic_simulator
FLUID_TIER=1 cargo build -p aerodynamic_simulator
```

The manifest declares `tier_0` through `tier_3` and depends on `core` plus `physics_core`.

## Known Limitations

- No verified public API exists yet; `src/lib.rs` is a stub.
- Coordinator-owned runtime config for this crate has not been added to `config/`.
- No open bug-pool item is tagged specifically to `components/aerodynamic_simulator`.
