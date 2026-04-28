# fem_structural - Usage Reference

## Architecture Overview

Current layout:

```text
fem_structural
|- Cargo.toml
\- src/lib.rs   init() stub
```

No solver, element, mesh, or material modules are present yet.

## Public API

```rust
pub fn init();
```

`init()` currently prints `fem_structural initialized`.

## Configuration

The expected `config/fem_structural.toml` file has not been created. Coordinator notes mention material defaults, solver tolerance, and maximum Newton iterations, but those configuration keys are [UNVERIFIED] until the file exists.

## Integration with Other Crates

This crate depends on `core` and `physics_core`, which matches the intended layering for SI units, rigid-body/world access, and shared numerical infrastructure.

```rust
use fem_structural::init;

fn main() {
    init();
}
```

## Numerical Details

The workspace physics contract requires implicit Newmark-beta or HHT-alpha style integration for stiff structural problems and sets Tier 1 as the minimum for basic FEM. The current crate source does not implement those numerical methods, so all solver details beyond the existence of `init()` are [UNVERIFIED - coordinator gate not yet published].

## Examples

Initialize the crate:

```rust
use fem_structural::init;

fn main() {
    init();
}
```

Shared-unit example:

```rust
use core::units::Meters;
use fem_structural::init;

fn main() {
    let _length = Meters(1.0);
    init();
}
```

## Troubleshooting

- If you expected mesh or solver APIs, they are not present yet.
- The current crate does not read any structural config file.
- Treat the current implementation as provisional until the review marker is cleared and C5 publishes its gate.
