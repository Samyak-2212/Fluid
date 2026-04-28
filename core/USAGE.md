# core - Usage Reference

## Architecture Overview

`core/src/lib.rs` exports the crate modules directly:

```text
core
|- ecs/          EntityId, Component, World, System, ArchetypeWorld
|- event_bus.rs  Event and EventBus traits
|- event_bus_impl.rs
|- math/         glam re-exports
|- threading/    ThreadPool and RayonPool
|- time/         fixed timestep manager
|- units.rs      SI newtypes
|- memory/       stub
\- scene/        stub
```

The crate is intentionally dependency-light. `glam` is re-exported through `core::math`, and `rayon` is wrapped by a trait so callers do not need to commit to a concrete pool implementation.

## Public API

```rust
pub struct EntityId(pub u64);
impl EntityId { pub fn raw(self) -> u64; }

pub trait Component: Send + Sync + 'static {}

pub trait World {
    fn spawn(&mut self) -> EntityId;
    fn insert<C: Component>(&mut self, entity: EntityId, component: C);
    fn get<C: Component>(&self, entity: EntityId) -> Option<&C>;
    fn get_mut<C: Component>(&mut self, entity: EntityId) -> Option<&mut C>;
    fn remove<C: Component>(&mut self, entity: EntityId);
    fn despawn(&mut self, entity: EntityId);
}

pub trait System<W: World>: Send + Sync {
    fn update(&mut self, world: &mut W, dt: Seconds);
}

pub struct ArchetypeWorld;
impl ArchetypeWorld {
    pub fn new() -> Self;
    pub fn entity_count(&self) -> usize;
}

pub trait Event: Send + Sync + 'static {}

pub trait EventBus: Send + Sync {
    fn publish<E: Event>(&self, event: E);
    fn subscribe<E: Event>(&self, handler: impl Fn(&E) + Send + Sync + 'static);
}

pub struct LocalEventBus;
impl LocalEventBus { pub fn new() -> Self; }

pub trait ThreadPool: Send + Sync {
    fn spawn<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;
}

pub struct RayonPool;
impl RayonPool { pub fn new() -> Self; }

pub struct Timestep;
impl Timestep {
    pub fn new() -> Self;
    pub fn dt(&self) -> Seconds;
    pub fn accumulated(&self) -> Seconds;
    pub fn add_frame_time(&mut self, frame_time: Seconds);
    pub fn tick(&mut self) -> bool;
}
```

`core::math` re-exports `glam::{DMat4, DQuat, DVec3, Mat3, Mat4, Quat, Vec2, Vec3, Vec4}`. `core::units` exports `Meters`, `Kilograms`, `Seconds`, `Newtons`, `Pascals`, `KilogramsPerCubicMeter`, `MetersPerSecond`, `MetersPerSecondSquared`, `Joules`, `Watts`, `Radians`, `RadiansPerSecond`, and `Kelvin`, each with `value() -> f64` plus arithmetic over `f64`.

## Configuration

`core` currently consumes `config/core.toml`.

| Key | Type | Default | Effect |
|---|---|---|---|
| `timestep_seconds` | `f64` | `0.016666666666666666` | Fixed simulation timestep used by `Timestep::new()`. |
| `max_ticks_per_frame` | `integer` | `8` | Spiral-of-death guard for callers that consume the config file. [UNVERIFIED: parsed directly in `core` runtime path not observed in current source.] |
| `rayon_num_threads` | `integer` | `0` | Thread-pool size override where `0` means Rayon decides. [UNVERIFIED: current `RayonPool` wrapper does not parse this key directly.] |

## Integration with Other Crates

`physics_core` depends on `core` for SI types and ECS identity. `rendering` depends on it for math, units, and world access.

```rust
use core::ecs::{ArchetypeWorld, World};
use core::units::Meters;

fn main() {
    let mut world = ArchetypeWorld::new();
    let entity = world.spawn();
    world.insert(entity, [Meters(0.0), Meters(1.0), Meters(2.0)]);
}
```

## Numerical Details

`core` establishes the numerical contract rather than solving equations itself. Physical quantities crossing module boundaries use SI newtypes, the timestep is expressed as `Seconds`, and Tier 0 is the only tier where Euler integration is allowed elsewhere in the workspace. `Timestep` defaults to `1/60 s` when configuration cannot be read.

## Examples

Using the ECS world:

```rust
use core::ecs::{ArchetypeWorld, World};

fn main() {
    let mut world = ArchetypeWorld::new();
    let entity = world.spawn();
    world.insert(entity, 42u32);
    assert_eq!(world.get::<u32>(entity), Some(&42));
}
```

Using the local event bus:

```rust
use core::event_bus::EventBus;
use core::event_bus_impl::LocalEventBus;

fn main() {
    let bus = LocalEventBus::new();
    bus.subscribe::<u32>(|value| assert_eq!(*value, 7));
    bus.publish(7u32);
}
```

## Troubleshooting

- If you need trait objects for world access, `World` is not dyn-compatible; use generic `W: World`.
- If `Timestep::new()` appears to ignore configuration, verify the process is started with `config/core.toml` reachable from the working directory.
- `scene` and `memory` are placeholders; avoid building downstream interfaces around them until they gain stable APIs.
