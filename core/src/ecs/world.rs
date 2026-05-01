// [NEEDS_REVIEW: claude]
//! Concrete archetype-based ECS world implementation.
//!
//! Provides [`ArchetypeWorld`], a concrete type implementing the [`WorldAny`] trait.
//! Internal storage uses a `HashMap<EntityId, HashMap<TypeId, Box<dyn Any + Send + Sync>>>`
//! (component-map-per-entity) layout.  This is deliberately simple: correctness and
//! interface coverage over performance.  A true archetype table (column-per-component,
//! contiguous memory) is a future Tier A optimisation and should not be attempted here.
//!
//! [`WorldAny`] is the object-safe erased trait implemented here. The typed [`World`]
//! extension methods (`insert<C>`, `get<C>`, `get_mut<C>`, `remove<C>`) are provided
//! automatically via the blanket `impl<T: WorldAny> World for T`.
//!
//! # Usage
//! ```rust,ignore
//! use core::ecs::{world::ArchetypeWorld, World};
//! let mut world = ArchetypeWorld::new();
//! let e = world.spawn();
//! world.insert(e, 42u32);
//! assert_eq!(world.get::<u32>(e), Some(&42));
//! ```

use std::any::{Any, TypeId};
use std::collections::HashMap;

use super::traits::{EntityId, WorldAny};

// ── Storage ──────────────────────────────────────────────────────────────────

type ComponentMap = HashMap<TypeId, Box<dyn Any + Send + Sync>>;

/// Concrete world backed by a map-of-maps storage model.
///
/// Every entity owns a `HashMap<TypeId, Box<dyn Any + Send + Sync>>` that stores
/// its components by type.  Lookup is O(1) average per component type per entity.
///
/// Implements [`WorldAny`] (object-safe, erased API). The typed [`World`] extension
/// API is automatically available via `impl<T: WorldAny> World for T`.
pub struct ArchetypeWorld {
    entities: HashMap<EntityId, ComponentMap>,
    next_id: u64,
}

impl ArchetypeWorld {
    /// Creates an empty world with no entities.
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1,
        }
    }

    /// Returns the number of live entities.
    #[inline]
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }
}

impl Default for ArchetypeWorld {
    fn default() -> Self {
        Self::new()
    }
}

// ── WorldAny impl ─────────────────────────────────────────────────────────────
//
// All type-erased methods live here. The typed `World` extension trait methods
// (`insert<C>`, `get<C>`, `get_mut<C>`, `remove<C>`) are default-implemented in
// `traits.rs` and require no code here.

// Safety: ArchetypeWorld owns its data exclusively and holds no raw pointers.
unsafe impl Send for ArchetypeWorld {}
unsafe impl Sync for ArchetypeWorld {}

impl WorldAny for ArchetypeWorld {
    fn spawn(&mut self) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        self.entities.insert(id, HashMap::new());
        id
    }

    fn despawn(&mut self, entity: EntityId) {
        self.entities.remove(&entity);
    }

    fn insert_erased(
        &mut self,
        entity: EntityId,
        type_id: TypeId,
        component: Box<dyn Any + Send + Sync>,
    ) {
        if let Some(map) = self.entities.get_mut(&entity) {
            map.insert(type_id, component);
        }
        // Silently ignore insert on a non-existent entity. Callers must
        // spawn before inserting. Panicking would be acceptable but is
        // more disruptive during early development.
    }

    fn get_erased(&self, entity: EntityId, type_id: TypeId) -> Option<&(dyn Any + Send + Sync)> {
        self.entities
            .get(&entity)?
            .get(&type_id)
            .map(|b| b.as_ref())
    }

    fn get_erased_mut(
        &mut self,
        entity: EntityId,
        type_id: TypeId,
    ) -> Option<&mut (dyn Any + Send + Sync)> {
        self.entities
            .get_mut(&entity)?
            .get_mut(&type_id)
            .map(|b| b.as_mut())
    }

    fn remove_erased(&mut self, entity: EntityId, type_id: TypeId) {
        if let Some(map) = self.entities.get_mut(&entity) {
            map.remove(&type_id);
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::traits::World;

    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    #[test]
    fn spawn_returns_unique_ids() {
        let mut world = ArchetypeWorld::new();
        let a = world.spawn();
        let b = world.spawn();
        assert_ne!(a, b);
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn insert_and_get() {
        let mut world = ArchetypeWorld::new();
        let e = world.spawn();
        world.insert(e, Position { x: 1.0, y: 2.0 });
        let p = world.get::<Position>(e).unwrap();
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
    }

    #[test]
    fn get_mut_modifies_component() {
        let mut world = ArchetypeWorld::new();
        let e = world.spawn();
        world.insert(e, Velocity { dx: 0.0, dy: 0.0 });
        world.get_mut::<Velocity>(e).unwrap().dx = 5.0;
        assert_eq!(world.get::<Velocity>(e).unwrap().dx, 5.0);
    }

    #[test]
    fn remove_component() {
        let mut world = ArchetypeWorld::new();
        let e = world.spawn();
        world.insert(e, Position { x: 0.0, y: 0.0 });
        world.remove::<Position>(e);
        assert!(world.get::<Position>(e).is_none());
    }

    #[test]
    fn despawn_removes_entity() {
        let mut world = ArchetypeWorld::new();
        let e = world.spawn();
        world.insert(e, Position { x: 0.0, y: 0.0 });
        world.despawn(e);
        assert_eq!(world.entity_count(), 0);
        assert!(world.get::<Position>(e).is_none());
    }

    #[test]
    fn multiple_component_types_on_one_entity() {
        let mut world = ArchetypeWorld::new();
        let e = world.spawn();
        world.insert(e, Position { x: 3.0, y: 4.0 });
        world.insert(e, Velocity { dx: 1.0, dy: -1.0 });
        assert_eq!(world.get::<Position>(e).unwrap().x, 3.0);
        assert_eq!(world.get::<Velocity>(e).unwrap().dx, 1.0);
    }

    #[test]
    fn overwrite_component() {
        let mut world = ArchetypeWorld::new();
        let e = world.spawn();
        world.insert(e, Position { x: 0.0, y: 0.0 });
        world.insert(e, Position { x: 9.0, y: 9.0 });
        assert_eq!(world.get::<Position>(e).unwrap().x, 9.0);
    }

    #[test]
    fn insert_on_despawned_entity_is_silent() {
        let mut world = ArchetypeWorld::new();
        let e = world.spawn();
        world.despawn(e);
        // Must not panic.
        world.insert(e, Position { x: 0.0, y: 0.0 });
    }

    /// Verify `ArchetypeWorld` is usable as `Box<dyn WorldAny>`.
    #[test]
    fn archetype_world_as_dyn_world_any() {
        let mut world: Box<dyn WorldAny> = Box::new(ArchetypeWorld::new());
        let e = world.spawn();
        world.insert_erased(e, TypeId::of::<Position>(), Box::new(Position { x: 1.0, y: 2.0 }));
        let any = world.get_erased(e, TypeId::of::<Position>()).unwrap();
        let p = any.downcast_ref::<Position>().unwrap();
        assert_eq!(p.x, 1.0);
        world.despawn(e);
    }
}
