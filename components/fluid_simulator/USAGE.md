# fluid_simulator - Usage Reference

## Architecture Overview

The crate currently consists of:

```text
fluid_simulator
|- Cargo.toml
\- src/lib.rs   stub comment only
```

Coordinator documentation indicates that future module splits are expected for SPH and CFD, but those modules are not present in source yet.

## Public API

No public Rust items are currently exported from `components/fluid_simulator/src/lib.rs`.

```rust
// Current source exports no pub structs, traits, or functions.
fn main() {}
```

## Configuration

`knowledge/config_schema.md` expects a future `config/fluid_simulator.toml` with SPH particle-cap and CFD resolution settings, but the file does not currently exist in the repository. That configuration remains [UNVERIFIED].

## Integration with Other Crates

The manifest declares dependencies on `core` and `physics_core`, which makes this crate part of the planned simulation layer above shared interfaces and integrators.

```rust
use fluid_simulator as _;

fn main() {}
```

Per `knowledge/dependency_graph.md`, this crate belongs to the C5 wave that follows `physics_core` interface publication.

## Numerical Details

Per `knowledge/physics_contract.md`, the intended fluid methods are SPH with the Wendland C2 kernel and XSPH correction plus CFD paths using projection methods or higher-order integrators. None of those algorithms are implemented in the current crate source, so this section is [UNVERIFIED - coordinator gate not yet published].

## Examples

Import-only smoke example:

```rust
use fluid_simulator as _;

fn main() {}
```

Dependency wiring example with other workspace crates:

```rust
use core::units::Seconds;
use fluid_simulator as _;

fn main() {
    let _dt = Seconds(0.016666666666666666);
}
```

## Troubleshooting

- If you expected SPH or CFD modules, they are not in the crate yet; only the manifest scaffold exists.
- If a design document mentions `config/fluid_simulator.toml`, treat it as planned rather than implemented until the file appears in the repository.
- Integration work should expect churn until C4 and C5 publish their gates.
