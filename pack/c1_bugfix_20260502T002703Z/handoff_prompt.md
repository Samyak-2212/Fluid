# Handoff: C1 BUG-001 Fix — ECS dyn World Object-Safety

Session ID: c1_bugfix_20260502T002703Z
Timestamp: 2026-05-02T00:27:03+05:30
Status: [RETIRED]

## Delivered

BUG-001 is CLOSED. `dyn WorldAny` is now a valid type in the Fluid framework.

## What changed

`core/src/ecs/` contains three files, all updated:

### traits.rs

Two-layer ECS world interface:

- `WorldAny` — object-safe erased trait. Methods: `spawn`, `despawn`,
  `insert_erased(EntityId, TypeId, Box<dyn Any+Send+Sync>)`,
  `get_erased(EntityId, TypeId) -> Option<&dyn Any+Send+Sync>`,
  `get_erased_mut`, `remove_erased`.
  Bounds: `: Send + Sync`. Fully dyn-compatible.

- `World: WorldAny` — typed extension supertrait. Methods `insert<C>`, `get<C>`,
  `get_mut<C>`, `remove<C>` are **default implementations** that delegate to the
  erased layer. Blanket impl: `impl<T: WorldAny> World for T {}`.
  Not object-safe by design.

### world.rs

`ArchetypeWorld` now implements `WorldAny`. The previous `impl World for ArchetypeWorld`
is replaced entirely by the blanket. Internal HashMap storage is unchanged.

### mod.rs

`WorldAny` re-exported alongside `World`.

## How to use dyn WorldAny

```rust
use core::ecs::{WorldAny, EntityId};
use std::any::TypeId;

fn store_world(w: Box<dyn WorldAny>) { ... }

// Typed access still available via World extension:
use core::ecs::World;
fn typed_insert<W: World>(w: &mut W, e: EntityId, v: f32) {
    w.insert(e, v); // calls insert_erased internally
}
```

## Test baseline

`cargo test -p core`: 28 passed, 0 failed.
`cargo check --workspace`: 0 errors.

## Successor notes

- Any new consumer that needs `dyn World` should use `dyn WorldAny` instead.
- `System<W: World>` continues to work unchanged — no migration needed.
- If a fully type-erased system dispatcher is needed, introduce a `SystemErased`
  trait backed by `dyn WorldAny` under Tier A review.
- The internal HashMap storage in `ArchetypeWorld` is still O(1) average per lookup.
  A true archetype layout (column-per-component) remains deferred Tier B work.
