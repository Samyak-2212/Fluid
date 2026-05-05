//! Scene graph — holds the simulation state via `Box<dyn WorldAny>`.
//!
//! The scene is the authoritative state container for:
//! - All simulation entities and their components (via `WorldAny`)
//! - Object transforms, visibility, and naming metadata
//! - Active camera and viewport settings
//!
//! # DEC-012
//! The ECS world is stored as `Box<dyn WorldAny>` (NOT `Box<dyn World>`).
//! `World` is not dyn-compatible — see BUG-001 resolution in `core/src/ecs/traits.rs`.
//!
//! # Mutation discipline (DEC-015)
//! All mutations to the scene MUST go through `CommandHistory::execute`.
//! Never call `Scene` mutation methods directly from the Iced update handler —
//! always wrap them in a `SceneCommand` impl.

pub mod command;

use std::collections::HashMap;
use fluid_core::ecs::traits::{EntityId, WorldAny};
use fluid_core::ecs::world::ArchetypeWorld;

// ── Position component ────────────────────────────────────────────────────────

/// World-space position of a scene entity, in metres.
///
/// Stored as `[f32; 3]` = [x, y, z] in right-handed Y-up coordinates.
/// Used by the viewport to position entity markers; will be superseded by a
/// full Transform component when the physics bridge is wired.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position(pub [f32; 3]);

impl Default for Position {
    fn default() -> Self {
        Self([0.0, 0.0, 0.0])
    }
}

/// A scene object's metadata (display name, visibility).
/// Component data lives inside the `WorldAny` store — this is UI-layer metadata only.
#[derive(Debug, Clone)]
pub struct ObjectMeta {
    pub name: String,
    pub visible: bool,
}

impl ObjectMeta {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), visible: true }
    }
}

/// The simulation scene graph.
///
/// Owns the ECS world (`Box<dyn WorldAny>`) and per-entity display metadata.
/// All mutations must go through `command::CommandHistory` (DEC-015).
pub struct Scene {
    /// Type-erased ECS world. Use `WorldAny` erased methods for cross-boundary storage.
    /// For typed component access in-process, downcast or use the `World` extension trait.
    world: Box<dyn WorldAny>,
    /// Per-entity display metadata (name, visibility).
    object_meta: HashMap<EntityId, ObjectMeta>,
    /// Ordered list of top-level entities for the scene outliner UI.
    root_entities: Vec<EntityId>,
    /// Name of the scene (shown in title bar and .fluid file envelope).
    pub name: String,
    /// Whether this scene has unsaved modifications.
    pub dirty: bool,
}

impl std::fmt::Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("name", &self.name)
            .field("entity_count", &self.root_entities.len())
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl Scene {
    /// Creates a new empty scene backed by `ArchetypeWorld`.
    pub fn new() -> Self {
        Self {
            world: Box::new(ArchetypeWorld::default()),
            object_meta: HashMap::new(),
            root_entities: Vec::new(),
            name: "Untitled".to_string(),
            dirty: false,
        }
    }

    /// Creates a scene backed by a provided `WorldAny` implementation.
    /// Use this in tests or when restoring from a serialized state.
    pub fn with_world(world: Box<dyn WorldAny>) -> Self {
        Self {
            world,
            object_meta: HashMap::new(),
            root_entities: Vec::new(),
            name: "Untitled".to_string(),
            dirty: false,
        }
    }

    /// Spawns a new entity with the given display name.
    ///
    /// Called by `SceneCommand` implementors — not directly from the update loop.
    pub fn spawn_object(&mut self, name: impl Into<String>) -> EntityId {
        let entity = self.world.spawn();
        self.object_meta.insert(entity, ObjectMeta::new(name));
        self.root_entities.push(entity);
        self.dirty = true;
        entity
    }

    /// Despawns an entity and removes its metadata.
    ///
    /// Called by `SceneCommand` implementors — not directly from the update loop.
    pub fn despawn_object(&mut self, entity: EntityId) {
        self.world.despawn(entity);
        self.object_meta.remove(&entity);
        self.root_entities.retain(|e| *e != entity);
        self.dirty = true;
    }

    /// Returns the display metadata for an entity, if it exists.
    pub fn meta(&self, entity: EntityId) -> Option<&ObjectMeta> {
        self.object_meta.get(&entity)
    }

    /// Returns mutable display metadata for an entity, if it exists.
    pub fn meta_mut(&mut self, entity: EntityId) -> Option<&mut ObjectMeta> {
        self.object_meta.get_mut(&entity)
    }

    /// Returns a read-only reference to the ECS world for viewport/sim queries.
    pub fn world(&self) -> &dyn WorldAny {
        self.world.as_ref()
    }

    /// Returns a mutable reference to the ECS world.
    ///
    /// Only call from within a `SceneCommand::execute` or `SceneCommand::undo`
    /// implementation. Do NOT call from Iced update handlers directly.
    pub fn world_mut(&mut self) -> &mut dyn WorldAny {
        self.world.as_mut()
    }

    /// Returns the ordered list of root entities (for the scene outliner).
    pub fn root_entities(&self) -> &[EntityId] {
        &self.root_entities
    }

    /// Returns the world-space position of `entity`, defaulting to origin if
    /// no `Position` component has been attached.
    pub fn get_position(&self, entity: EntityId) -> [f32; 3] {
        // Use `World` typed extension (blanket-impl on ArchetypeWorld via WorldAny).
        // We need to reach the concrete world — we can only call WorldAny erased
        // methods here since `self.world` is `Box<dyn WorldAny>`.
        use std::any::TypeId;
        self.world
            .get_erased(entity, TypeId::of::<Position>())
            .and_then(|any| any.downcast_ref::<Position>())
            .map(|p| p.0)
            .unwrap_or([0.0, 0.0, 0.0])
    }

    /// Sets the world-space position of `entity`.
    ///
    /// Call only from within a `SceneCommand` implementation (DEC-015).
    pub fn set_position(&mut self, entity: EntityId, pos: [f32; 3]) {
        use std::any::TypeId;
        self.world.insert_erased(
            entity,
            TypeId::of::<Position>(),
            Box::new(Position(pos)),
        );
        self.dirty = true;
    }

    /// Collects `(EntityId, [f32; 3])` world-space positions for every root
    /// entity, in outliner order, capped at `limit`.
    ///
    /// Entities without a `Position` component default to `[0, 0, 0]`.
    pub fn entity_positions(&self, limit: usize) -> Vec<(EntityId, [f32; 3])> {
        self.root_entities
            .iter()
            .take(limit)
            .map(|&e| (e, self.get_position(e)))
            .collect()
    }

    /// Marks the scene as clean (saved to disk).
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Marks the scene as having unsaved modifications.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scene_is_empty() {
        let s = Scene::new();
        assert_eq!(s.root_entities().len(), 0);
        assert_eq!(s.name, "Untitled");
        assert!(!s.dirty);
    }

    #[test]
    fn spawn_and_despawn() {
        let mut s = Scene::new();
        let e = s.spawn_object("Cube");
        assert_eq!(s.root_entities().len(), 1);
        assert!(s.dirty);
        assert_eq!(s.meta(e).unwrap().name, "Cube");

        s.despawn_object(e);
        assert_eq!(s.root_entities().len(), 0);
        assert!(s.meta(e).is_none());
    }

    #[test]
    fn mark_clean_clears_dirty() {
        let mut s = Scene::new();
        s.spawn_object("A");
        assert!(s.dirty);
        s.mark_clean();
        assert!(!s.dirty);
    }
}
