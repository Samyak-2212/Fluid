# motion_force_simulator

Planned component crate for force application, actuators, and joint-driven motion.

## What It Does

`motion_force_simulator` is intended to sit above `physics_core` and apply external forces or actuator-driven motion to ECS-managed bodies. The coordinator spec assigns gravity, springs, motors, and joint actuation behavior to this crate.

In current verified source, the crate consists of a manifest plus a stub `src/lib.rs` comment. The project manifest still lists the simulation-components domain as blocked, so the implementation contract has not yet been realized in code.

Any description of actuator models or joint semantics beyond this scaffold is [UNVERIFIED - coordinator gate not yet published].

## Capability Tier

| Feature area | Minimum tier | Status |
|---|---|---|
| Basic force accumulation | 0 | [UNVERIFIED - coordinator gate not yet published] |
| Extended actuator behavior | 1 | [UNVERIFIED - coordinator gate not yet published] |
| Advanced coupled systems | 2 | [UNVERIFIED - coordinator gate not yet published] |

## Quick Start

```toml
[dependencies]
motion_force_simulator = { path = "../components/motion_force_simulator" }
```

```rust
use motion_force_simulator as _;

fn main() {}
```

```bash
cargo build -p motion_force_simulator
```

## Build Instructions

```bash
cargo build -p motion_force_simulator
FLUID_TIER=0 cargo build -p motion_force_simulator
```

The crate declares all four tier features and depends on `core` plus `physics_core`.

## Known Limitations

- The crate exports no verified runtime API yet.
- Planned config in `config/motion_force_simulator.toml` is not present.
- No open bug-pool entry is currently tagged specifically to `components/motion_force_simulator`.
