# thermodynamic_simulator - Usage Reference

## Architecture Overview

Current layout:

```text
thermodynamic_simulator
|- Cargo.toml
\- src/lib.rs   init() stub
```

No internal modules are present yet.

## Public API

```rust
pub fn init();
```

`init()` currently prints `thermodynamic_simulator initialized` and does not accept configuration or state inputs.

## Configuration

The expected `config/thermodynamic_simulator.toml` file is not present in the repository. The coordinator prompt names heat-capacity and conductivity defaults as planned inputs, but those remain [UNVERIFIED].

## Integration with Other Crates

The manifest declares dependencies on `core` and `physics_core`, which places this crate in the thermal-simulation layer above shared units and integrator infrastructure.

```rust
use thermodynamic_simulator::init;

fn main() {
    init();
}
```

## Numerical Details

The workspace physics contract calls for operator splitting with RK4 for thermodynamics, and the C5 prompt expects temperature to be represented with `core::units::Kelvin`. The current crate source does not implement those algorithms, so all numerical behavior beyond the existence of `init()` is [UNVERIFIED - coordinator gate not yet published].

## Examples

Initialize the crate:

```rust
use thermodynamic_simulator::init;

fn main() {
    init();
}
```

Pair with shared units:

```rust
use core::units::Kelvin;
use thermodynamic_simulator::init;

fn main() {
    let _ambient = Kelvin(293.15);
    init();
}
```

## Troubleshooting

- If you expected temperature-state APIs, they are not present yet.
- The crate currently does not parse any config file.
- The file-level review marker indicates the implementation should be treated as provisional.
