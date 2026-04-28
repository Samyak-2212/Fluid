# aerodynamic_simulator - Usage Reference

## Architecture Overview

The crate currently contains only a manifest and stub library entry point:

```text
aerodynamic_simulator
|- Cargo.toml
\- src/lib.rs   stub comment only
```

Expected aerodynamic submodules are not present yet.

## Public API

No public items are exported by the current `src/lib.rs`.

```rust
fn main() {}
```

## Configuration

`knowledge/config_schema.md` and the C5 coordinator prompt describe a future `config/aerodynamic_simulator.toml` for air density, viscosity, and reference-area inputs. That file is absent, so the configuration contract is currently [UNVERIFIED].

## Integration with Other Crates

The manifest positions this crate above `core` and `physics_core`.

```rust
use aerodynamic_simulator as _;

fn main() {}
```

The intended dependency flow is `core` -> `physics_core` -> `aerodynamic_simulator`, but the force-model layer has not been implemented yet.

## Numerical Details

The repository-level physics contract requires SI units and tier-specific numerical behavior. The aerodynamic equations themselves are not implemented in source, so any detailed aerodynamic model description remains [UNVERIFIED - coordinator gate not yet published].

## Examples

Basic import:

```rust
use aerodynamic_simulator as _;

fn main() {}
```

Workspace wiring:

```rust
use core::units::KilogramsPerCubicMeter;
use aerodynamic_simulator as _;

fn main() {
    let _rho = KilogramsPerCubicMeter(1.225);
}
```

## Troubleshooting

- If you expected lift/drag APIs, they are not committed yet.
- If a config file is referenced externally, confirm whether it exists locally before integrating around it.
- Expect interface churn until the simulation-components gate is published.
