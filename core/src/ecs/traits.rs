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
//!
//! # Object-safety (BUG-001 fix)
//! `World` has generic methods which are not dyn-compatible. The fix splits the
//! interface into two layers:
//!
//! - [`WorldAny`] — object-safe, type-erased; suitable for `Box<dyn WorldAny>`.
//! - [`World`] — typed extension trait that adds generic convenience methods as
//!   default implementations on top of [`WorldAny`]. Not object-safe, but
//!   generic usage (`W: World`) works everywhere it was used before.
//!
//! Implementors write [`WorldAny`] only. [`World`] is provided via a blanket impl
//! for any `T: WorldAny`.

use std::any::{Any, TypeId};

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

// ── WorldAny — object-safe erased world interface ────────────────────────────

/// Object-safe, type-erased world interface.
///
/// All methods use [`TypeId`] and [`Any`] trait objects instead of generic
/// type parameters, making this trait dyn-compatible. Use this when you need
/// `Box<dyn WorldAny>` or `&dyn WorldAny`.
///
/// For ergonomic typed access, use the [`World`] extension trait, which is
/// automatically implemented for every `T: WorldAny`.
pub trait WorldAny: Send + Sync {
    /// Creates a new entity with no components. Returns its unique [`EntityId`].
    fn spawn(&mut self) -> EntityId;

    /// Destroys an entity and all its attached components.
    fn despawn(&mut self, entity: EntityId);

    /// Attaches a type-erased component to `entity`.
    ///
    /// `type_id` must equal `TypeId::of::<C>()` for the concrete component `C`
    /// stored in `component`. Implementations may silently drop components whose
    /// `type_id` does not match the box contents, but this is a caller error.
    fn insert_erased(
        &mut self,
        entity: EntityId,
        type_id: TypeId,
        component: Box<dyn Any + Send + Sync>,
    );

    /// Returns a shared reference to the stored component identified by
    /// `type_id`, if present on `entity`.
    fn get_erased(&self, entity: EntityId, type_id: TypeId) -> Option<&(dyn Any + Send + Sync)>;

    /// Returns an exclusive reference to the stored component identified by
    /// `type_id`, if present on `entity`.
    fn get_erased_mut(
        &mut self,
        entity: EntityId,
        type_id: TypeId,
    ) -> Option<&mut (dyn Any + Send + Sync)>;

    /// Removes the component identified by `type_id` from `entity`, if present.
    fn remove_erased(&mut self, entity: EntityId, type_id: TypeId);
}

// ── World — typed extension trait ────────────────────────────────────────────

/// Typed extension trait that adds generic convenience methods over [`WorldAny`].
///
/// This trait is **not** dyn-compatible (its methods have generic type parameters),
/// but it is automatically implemented for every `T: WorldAny` via a blanket impl,
/// so no manual implementation is required.
///
/// Use `W: World` as a type parameter when you need typed component access.
/// Use `dyn WorldAny` when you need type erasure (e.g. storing heterogeneous worlds).
///
/// # Why two traits?
/// [`WorldAny`] must be object-safe (no generic methods). Adding typed convenience
/// methods requires generic type parameters, which breaks dyn-compatibility. The two-
/// layer design gives both: type-erased `dyn WorldAny` for storage/dispatch, and
/// ergonomic generic `World` methods for system code. This is the accepted fix for
/// BUG-001 (C1 reactivation, 2026-05-02).
pub trait World: WorldAny {
    /// Attaches a component of type `C` to `entity`.
    ///
    /// Overwrites any previously attached component of the same type.
    #[inline]
    fn insert<C: Component>(&mut self, entity: EntityId, component: C) {
        self.insert_erased(entity, TypeId::of::<C>(), Box::new(component));
    }

    /// Returns a shared reference to the `C` component of `entity`, if present.
    #[inline]
    fn get<C: Component>(&self, entity: EntityId) -> Option<&C> {
        self.get_erased(entity, TypeId::of::<C>())
            .and_then(|any| any.downcast_ref::<C>())
    }

    /// Returns an exclusive reference to the `C` component of `entity`, if present.
    #[inline]
    fn get_mut<C: Component>(&mut self, entity: EntityId) -> Option<&mut C> {
        self.get_erased_mut(entity, TypeId::of::<C>())
            .and_then(|any| any.downcast_mut::<C>())
    }

    /// Removes the `C` component from `entity`, if present.
    #[inline]
    fn remove<C: Component>(&mut self, entity: EntityId) {
        self.remove_erased(entity, TypeId::of::<C>());
    }
}

/// Blanket impl: every `T: WorldAny` automatically gets the typed `World` API.
impl<T: WorldAny> World for T {}

// ── System trait ─────────────────────────────────────────────────────────────

/// A system is a unit of logic that transforms world state each simulation step.
///
/// Systems receive a mutable world reference and the elapsed time `dt`.
/// The `Send + Sync` bounds allow the scheduler to run systems in parallel
/// (Tier 1+) without unsafe code.
///
/// # Why a type parameter instead of `dyn WorldAny`
/// `World` (the typed extension) has generic methods and is not dyn-compatible.
/// `WorldAny` is object-safe and can be used as `dyn WorldAny`, but systems
/// written against the typed `World` API must use `W: World`. Using `W: World`
/// as a type parameter preserves the full generic interface while keeping `System`
/// itself object-safe for a known `W` (a concrete `Box<dyn System<W>>` can be
/// stored). If a fully type-erased system scheduler is needed in a future revision,
/// introduce a `SystemErased` trait backed by `dyn WorldAny` under Tier A review.
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

    // Compile-time check: WorldAny is object-safe (dyn WorldAny must compile).
    #[test]
    fn world_any_is_dyn_compatible() {
        fn accepts_dyn(_w: &dyn WorldAny) {}
        fn accepts_box(_w: Box<dyn WorldAny>) {}
        // If WorldAny were not dyn-compatible these function signatures would
        // fail to compile. This test is a compile-time gate.
        let _ = accepts_dyn as fn(&dyn WorldAny);
        let _ = accepts_box as fn(Box<dyn WorldAny>);
    }
}
