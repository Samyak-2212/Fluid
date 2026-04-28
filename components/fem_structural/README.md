# fem_structural

Early finite-element structural component crate with a single initialization entry point.

## What It Does

`fem_structural` is the workspace crate intended for structural finite-element analysis. The coordinator spec assigns linear and nonlinear FEM solving to it and expects integration with `physics_core` plus runtime material configuration.

Verified source is still minimal. The crate exports only `pub fn init()` from `src/lib.rs`, and that file is tagged `[NEEDS_REVIEW: claude]`. The broader simulation-components coordinator is still blocked in the project manifest, so solver behavior described outside the current source must be treated cautiously.

At present this crate is best understood as a placeholder for future FEM work with a minimal smoke-test function.

## Capability Tier

| Feature area | Minimum tier | Status |
|---|---|---|
| Linear FEM | 1 | [UNVERIFIED - coordinator gate not yet published] |
| Nonlinear Newton-Raphson FEM | 2 | [UNVERIFIED - coordinator gate not yet published] |
| Tier 3 compute acceleration | 3 | [UNVERIFIED - coordinator gate not yet published] |

## Quick Start

```toml
[dependencies]
fem_structural = { path = "../components/fem_structural" }
```

```rust
use fem_structural::init;

fn main() {
    init();
}
```

```bash
cargo build -p fem_structural
```

## Build Instructions

```bash
cargo build -p fem_structural
FLUID_TIER=1 cargo build -p fem_structural
FLUID_TIER=2 cargo build -p fem_structural
```

The crate declares all tier features and depends on `core` plus `physics_core`.

## Known Limitations

- The only verified public API is `pub fn init()`.
- The source file is marked `[NEEDS_REVIEW: claude]`.
- No runtime config file for material defaults or solver tolerances exists yet.
