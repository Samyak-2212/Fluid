# motion_force_simulator - Usage Reference

## Architecture Overview

Current on-disk structure:

```text
motion_force_simulator
|- Cargo.toml
\- src/lib.rs   stub comment only
```

Force models, actuators, and joint modules have not been added yet.

## Public API

No public Rust API is exported by the current source file.

```rust
fn main() {}
```

## Configuration

The coordinator prompt expects a future `config/motion_force_simulator.toml` for actuator models and gravity-vector configuration. That file does not exist yet, so configuration details are [UNVERIFIED].

## Integration with Other Crates

This crate is meant to modify `physics_core` rigid-body state while using `core` types for units and ECS wiring. Only the dependency declaration is verified today.

```rust
use motion_force_simulator as _;

fn main() {}
```

## Numerical Details

Per the workspace physics contract, force application should operate in SI units and leave integration itself to `physics_core`. The concrete numerical behavior is not implemented in the current crate source and is therefore [UNVERIFIED - coordinator gate not yet published].

## Examples

Import-only example:

```rust
use motion_force_simulator as _;

fn main() {}
```

Shared-units example:

```rust
use core::units::Newtons;
use motion_force_simulator as _;

fn main() {
    let _force = Newtons(10.0);
}
```

## Troubleshooting

- If you expected actuator or joint APIs, they are not present in the repository yet.
- Planned runtime configuration is not implemented; avoid depending on config keys that only appear in coordinator prompts.
- Recheck `knowledge/project_manifest.md` before integrating this crate into a larger workflow.
