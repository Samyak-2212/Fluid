// [NEEDS_REVIEW: claude]
//! Concrete archetype-based ECS world implementation.
//!
//! Provides [`ArchetypeWorld`], a concrete type implementing the [`World`] trait.
//! Internal storage uses a `HashMap<EntityId, HashMap<TypeId, Box<dyn Any + Send + Sync>>>`
//! (component-map-per-entity) layout.  This is deliberately simple: correctness and
//! interface coverage over performance.  A true archetype table (column-per-component,
//! contiguous memory) is a future Tier A optimisation and should not be attempted here.
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

use super::traits::{Component, EntityId, World};

// ── Storage ──────────────────────────────────────────────────────────────────

type ComponentMap = HashMap<TypeId, Box<dyn Any + Send + Sync>>;

/// Concrete world backed by a map-of-maps storage model.
///
/// Every entity owns a `HashMap<TypeId, Box<dyn Any + Send + Sync>>` that stores
/// its components by type.  Lookup is O(1) average per component type per entity.
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

// ── World impl ───────────────────────────────────────────────────────────────

impl World for ArchetypeWorld {
    fn spawn(&mut self) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        self.entities.insert(id, HashMap::new());
        id
    }

    fn insert<C: Component>(&mut self, entity: EntityId, component: C) {
        if let Some(map) = self.entities.get_mut(&entity) {
            map.insert(TypeId::of::<C>(), Box::new(component));
        }
        // Silently ignore insert on a non-existent entity.  Callers must
        // spawn before inserting.  Panicking would be acceptable but is
        // more disruptive during early development.
    }

    fn get<C: Component>(&self, entity: EntityId) -> Option<&C> {
        self.entities
            .get(&entity)?
            .get(&TypeId::of::<C>())?
            .downcast_ref::<C>()
    }

    fn get_mut<C: Component>(&mut self, entity: EntityId) -> Option<&mut C> {
        self.entities
            .get_mut(&entity)?
            .get_mut(&TypeId::of::<C>())?
            .downcast_mut::<C>()
    }

    fn remove<C: Component>(&mut self, entity: EntityId) {
        if let Some(map) = self.entities.get_mut(&entity) {
            map.remove(&TypeId::of::<C>());
        }
    }

    fn despawn(&mut self, entity: EntityId) {
        self.entities.remove(&entity);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
}
