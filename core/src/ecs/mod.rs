//! ECS module — re-exports traits and the `EntityId` type.

pub mod traits;
pub mod world;

pub use traits::EntityId;
pub use traits::{Component, System, World, WorldAny};
pub use world::ArchetypeWorld;
// Note: System is generic over W: World — import as System<MyWorldType>.
// WorldAny is the object-safe subset; use dyn WorldAny for type-erased world storage.
