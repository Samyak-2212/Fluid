//! ECS component, system, and world traits.
//!
//! This module defines the trait surface that all ECS consumers depend on.
//! Concrete storage (archetype layout, sparse sets, etc.) is Tier B work — deferred.
//! The trait layer here is the Tier A deliverable for [C1_INTERFACES_PUBLISHED].
//!
//! # Entity model
//! Every live object in the simulation is an [`EntityId`].
//! Components are plain data attached to an entity via a [`World`] implementation.
//! Systems operate on a mutable [`World`] reference each tick.

use crate::units::Seconds;

// ── Entity identifier ────────────────────────────────────────────────────────

/// Opaque handle for a simulation entity.
///
/// Wraps a `u64` to prevent accidental arithmetic on IDs.
/// Generation bits (for slot reuse detection) may be added in a future revision;
/// expose only `raw()` for now so the layout can change without breaking callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub u64);

impl EntityId {
    /// Returns the raw numeric value of the ID.
    #[inline]
    pub fn raw(self) -> u64 {
        self.0
    }
}

// ── Component trait ──────────────────────────────────────────────────────────

/// Marker trait for data that can be stored on an entity.
///
/// Any `Send + Sync + 'static` type may implement this; no methods required.
/// The bounds ensure components are safe to pass across threads (required for
/// the parallel system execution that higher tiers enable).
pub trait Component: Send + Sync + 'static {}

/// Blanket implementation: any `T: Send + Sync + 'static` is a `Component`.
///
/// Concrete crates may opt out by adding a negative impl (nightly only).
/// For stable Rust, requiring explicit `impl Component for MyType {}` is also
/// acceptable — see PROMPT.md discussion on ECS design.
impl<T: Send + Sync + 'static> Component for T {}

// ── World trait ──────────────────────────────────────────────────────────────

/// Abstract container that owns all live entities and their component data.
///
/// Implementations are free to choose their internal representation
/// (archetype table, sparse sets, etc.) as long as this interface is satisfied.
pub trait World {
    /// Creates a new entity with no components. Returns its unique [`EntityId`].
    fn spawn(&mut self) -> EntityId;

    /// Attaches a component of type `C` to `entity`.
    ///
    /// Overwrites any previously attached component of the same type.
    fn insert<C: Component>(&mut self, entity: EntityId, component: C);

    /// Returns a shared reference to the `C` component of `entity`, if present.
    fn get<C: Component>(&self, entity: EntityId) -> Option<&C>;

    /// Returns an exclusive reference to the `C` component of `entity`, if present.
    fn get_mut<C: Component>(&mut self, entity: EntityId) -> Option<&mut C>;

    /// Removes the `C` component from `entity`, if present.
    fn remove<C: Component>(&mut self, entity: EntityId);

    /// Destroys an entity and all its attached components.
    fn despawn(&mut self, entity: EntityId);
}

// ── System trait ─────────────────────────────────────────────────────────────

/// A system is a unit of logic that transforms world state each simulation step.
///
/// Systems receive a mutable world reference and the elapsed time `dt`.
/// The `Send + Sync` bounds allow the scheduler to run systems in parallel
/// (Tier 1+) without unsafe code.
///
/// # Why a type parameter instead of `dyn World`
/// [`World`] has generic methods (`insert`, `get`, `get_mut`, `remove`) which
/// make it dyn-incompatible in Rust's object model. Using `W: World` as a
/// type parameter preserves the full generic interface while keeping `System`
/// itself object-safe (a concrete `Box<dyn System<W>>` can be stored for a
/// known `W`). If a type-erased world is required in a future revision, a
/// separate `AnyWorld` trait backed by `Any` downcasting should be introduced
/// under Tier A review.
pub trait System<W: World>: Send + Sync {
    /// Executes one step of this system's logic.
    ///
    /// `dt` is the fixed simulation timestep in SI seconds, sourced from
    /// `config/core.toml` via the time-step manager. Systems must not
    /// read wall-clock time directly.
    fn update(&mut self, world: &mut W, dt: Seconds);
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_id_raw() {
        let id = EntityId(42);
        assert_eq!(id.raw(), 42);
    }

    #[test]
    fn entity_id_equality() {
        assert_eq!(EntityId(1), EntityId(1));
        assert_ne!(EntityId(1), EntityId(2));
    }

    // Compile-time check: EntityId is Copy.
    #[test]
    fn entity_id_copy() {
        let a = EntityId(7);
        let b = a; // copy
        assert_eq!(a, b);
    }

    // Verify the blanket Component impl applies.
    #[allow(dead_code)]
    struct TestComp(i32);
    // TestComp: Send + Sync + 'static — blanket impl applies automatically.

    #[test]
    fn test_comp_is_component() {
        fn assert_component<C: Component>() {}
        assert_component::<TestComp>();
    }
}
