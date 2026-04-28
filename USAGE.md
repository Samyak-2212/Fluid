# Fluid - Usage Reference

## Architecture Overview

The workspace root organizes the project into independent crates:

```text
Fluid workspace
|- core/                 foundational ECS, units, threading, time
|- physics_core/         integrators, collision, constraints, rigid bodies
|- rendering/            wgpu and Tier 0 software rendering
|- builder/              local build UI
|- debugger/             localhost debugger surface
\- components/           domain simulators
   |- fluid_simulator/
   |- aerodynamic_simulator/
   |- motion_force_simulator/
   |- thermodynamic_simulator/
   \- fem_structural/
```

The root `Cargo.toml` defines workspace members, shared dependency versions, and build profiles. It does not expose a Rust library target.

## Public API

The workspace root has no public Rust API. Downstream users consume member crates directly.

```rust
// No crate root library is exported from the workspace manifest.
// Depend on a member crate such as `core`, `rendering`, or `physics_core`.
fn main() {}
```

## Configuration

Workspace-level configuration is distributed across `config/` files rather than a single root config file.

| File | Keys currently present | Effect |
|---|---|---|
| `config/builder_flags.toml` | `[[flag]]` entries with `name`, `kind`, `label`, `description`, `type`, `options`, `default` | Drives the builder UI and Cargo invocation shaping. |
| `config/core.toml` | `timestep_seconds`, `max_ticks_per_frame`, `rayon_num_threads` | Controls fixed timestep and thread-pool behavior in `core`. |
| `config/rendering.toml` | `[preview]`, `[frame]`, `[camera]`, `[debug_overlay]` keys | Controls preview port, frame size, camera defaults, and release overlay behavior. |
| `config/physics_core.toml` | `constraint_solver_iterations`, `broadphase_cell_size` | Controls solver iteration count and broadphase grid sizing. |

Additional component config files are expected by coordinator spec but do not all exist yet. Those planned files remain [UNVERIFIED].

## Integration with Other Crates

Integration happens by selecting member crates directly:

```rust
use core::ecs::ArchetypeWorld;
use rendering::{Camera, StubRenderer, SceneRenderer};

fn main() {
    let world = ArchetypeWorld::new();
    let camera = Camera::default_perspective(16.0 / 9.0);
    let mut renderer = StubRenderer::new();
    renderer.render(&world, &camera);
}
```

Per `knowledge/dependency_graph.md`, `core` unblocks `rendering`, `physics_core`, and `debugger`, while `physics_core` interfaces are meant to unblock the component crates.

## Numerical Details

<!-- This section intentionally omitted: not a physics or rendering crate. -->

## Examples

Current `core` usage from the workspace:

```rust
use core::time::Timestep;

fn main() {
    let timestep = Timestep::new();
    let _dt = timestep.dt();
}
```

Import-only example for a partially implemented crate:

```rust
use rendering as _;

fn main() {}
```

## Troubleshooting

- `cargo build` at the workspace root may still be affected by `BUG-001` in `core/ecs`.
- If a crate appears under the workspace but exposes almost no API, check `knowledge/project_manifest.md`; several coordinators have not yet published completion gates.
- If `FLUID_TIER` behavior looks inconsistent, remember it is compile-time only and changing the value requires a rebuild.
