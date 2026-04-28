//! `core` crate — foundational interfaces for the Fluid framework.
//!
//! # Module layout
//! - [`units`]       — SI newtype wrappers (consumed by `physics_core`)
//! - [`ecs`]         — ECS entity, component, system, world traits + concrete [`ecs::ArchetypeWorld`]
//! - [`event_bus`]   — publish/subscribe event bus trait
//! - [`event_bus_impl`] — concrete [`event_bus_impl::LocalEventBus`] implementation
//! - [`math`]        — math primitives (re-exports from `glam` 0.32.1)
//! - [`time`]        — fixed-timestep manager
//! - [`threading`]   — thread pool trait + concrete [`threading::RayonPool`]
//! - [`memory`]      — custom allocator interfaces (stub; post-gate Tier B work)
//! - [`scene`]       — scene graph (stub; post-gate Tier B work)

pub mod ecs;
pub mod event_bus;
pub mod event_bus_impl;
pub mod math;
pub mod memory;
pub mod scene;
pub mod threading;
pub mod time;
pub mod units;
