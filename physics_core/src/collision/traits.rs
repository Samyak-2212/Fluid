//! Collision detection trait surface for the Fluid physics core.
//!
//! # Pipeline overview
//!
//! ```text
//! [Broadphase]              → candidate entity pairs
//!     ↓
//! [GJK narrowphase]         → boolean: shapes intersect?
//!     ↓ (if intersecting)
//! [EPA penetration query]   → ContactManifold { depth, normal, point }
//!     ↓
//! [ConstraintSolver]        → resolves penetration via impulses
//! ```
//!
//! Concrete implementations:
//! - `gjk.rs`         — GJK intersection test
//! - `epa.rs`         — EPA penetration depth/normal extraction
//! - `broadphase.rs`  — spatial-hash broadphase (Tier 0+)
//!
//! All quantities crossing public API boundaries must use SI newtype wrappers
//! from `core::units`. Raw `f64` for physical quantities is a bug.

use core::{ecs::EntityId, units::Meters};
use glam::Vec3;

// ── Convex shape support function ─────────────────────────────────────────────

/// A convex geometry that can supply a support point in any direction.
///
/// The GJK algorithm interacts with shapes exclusively through this trait,
/// making it agnostic to the underlying geometry representation (sphere,
/// capsule, convex hull, etc.).
///
/// # Contract
///
/// `support(d)` must return the point **p** on or inside the shape such that
/// `p · d` is maximised. The returned point is in world space.
/// `d` need not be normalised; the implementation must handle any non-zero
/// direction vector.
pub trait ConvexShape: Send + Sync {
    /// Returns the support point of this shape in direction `direction`.
    ///
    /// `direction` is a world-space vector (not required to be unit length).
    fn support(&self, direction: Vec3) -> Vec3;
}

// ── Shape reference ───────────────────────────────────────────────────────────

/// A typed reference to a convex shape paired with its owning entity.
///
/// Passed as a slice to [`CollisionDetector::detect`] so the detector can
/// report which entity pair generated each contact.
pub struct ShapeRef<'a> {
    /// The entity that owns this shape.
    pub entity: EntityId,
    /// The convex shape geometry.
    pub shape: &'a dyn ConvexShape,
}

// ── Contact manifold ──────────────────────────────────────────────────────────

/// Describes the contact geometry between two overlapping shapes.
///
/// Produced by the narrowphase (GJK + EPA) pipeline.
/// Consumed by the constraint solver to generate corrective impulses.
///
/// # Unit conventions
///
/// - `depth` is in SI metres, wrapped as [`Meters`].
/// - `normal` and `contact_point` are dimensionless/world-space vectors;
///   their physical scale is implied by the simulation's metre-per-unit
///   convention. Comments on each field document the intent.
#[derive(Debug, Clone)]
pub struct ContactManifold {
    /// Entity A involved in the contact.
    pub entity_a: EntityId,
    /// Entity B involved in the contact.
    pub entity_b: EntityId,
    /// World-space point at which the contact occurs (metres from origin,
    /// stored as raw `Vec3` per the glam-is-unitless convention).
    pub contact_point: Vec3,
    /// Contact normal pointing from B toward A (unit vector).
    pub normal: Vec3,
    /// Penetration depth along `normal` in SI metres.
    pub depth: Meters,
}

// ── Collision detector ────────────────────────────────────────────────────────

/// Performs narrowphase collision detection over a set of shapes and returns
/// all contacts found in this step.
///
/// Implementations are responsible for running GJK on each candidate pair
/// and invoking EPA when GJK reports an intersection.
///
/// # Usage
///
/// The broadphase (see `broadphase.rs`) filters the full shape set down to
/// candidate pairs. The [`CollisionDetector`] then tests each pair precisely.
/// In practice the caller may compose broadphase + narrowphase into a single
/// pipeline step.
pub trait CollisionDetector: Send + Sync {
    /// Tests all shapes in `shapes` for pairwise intersection and returns
    /// one [`ContactManifold`] per intersecting pair.
    ///
    /// The caller must ensure that `shapes` contains only shapes whose
    /// AABBs overlap (i.e., the broadphase has already been applied).
    fn detect(&self, shapes: &[ShapeRef<'_>]) -> Vec<ContactManifold>;
}

// ── Broadphase interface ──────────────────────────────────────────────────────

/// Broadphase collision system: cheaply filters the full set of shapes down
/// to candidate pairs that may be intersecting.
///
/// The concrete implementation in `broadphase.rs` uses a spatial hash grid.
/// BVH construction is not used at Tier 0 (too expensive to rebuild per frame).
pub trait Broadphase: Send + Sync {
    /// Returns index pairs `(i, j)` into the `shapes` slice where entity `i`
    /// and entity `j` are in the same spatial hash cell and warrant a
    /// narrowphase test. `i < j` is guaranteed.
    fn candidate_pairs(&self, shapes: &[ShapeRef<'_>]) -> Vec<(usize, usize)>;
}
