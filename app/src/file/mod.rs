//! File format — `.fluid` MessagePack envelope (C8-FileFormat, DEC-004 + DEC-011).
//!
//! # Format
//!
//! `.fluid` files are MessagePack documents using **map-based** struct serialization
//! (DEC-011). Map-based encoding means field names are stored as keys, making the
//! format resilient to field additions and reordering.
//!
//! Top-level envelope shape:
//!
//! ```text
//! { "format_version": 1, "app_version": "0.1.0", "scene_name": "...", "entities": [...] }
//! ```
//!
//! # History note
//! Session 4's handoff_prompt.md incorrectly cited DEC-003 as specifying TOML.
//! Session 5 implemented TOML accordingly. Session 6 reverts to MessagePack per
//! user-approved Option A (DEC-004 + DEC-011 are LOCKED; see BUG-020).
//! The `FluidEnvelope` struct is unchanged — only the codec (TOML → rmp-serde).

use std::path::Path;

use serde::{Deserialize, Serialize};

// ── Wire types ────────────────────────────────────────────────────────────────

/// Envelope for a `.fluid` scene file.
///
/// Serialized with `rmp_serde::to_vec_named` (map-based, DEC-011).
/// Keep this struct serde-compatible with any future codec swap —
/// do not use `#[serde(rename_all)]` or array-packed attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FluidEnvelope {
    /// Increment this when the schema changes so loaders can migrate.
    pub format_version: u32,
    /// Application version that wrote this file (e.g. "0.1.0").
    pub app_version: String,
    /// Display name of the scene.
    pub scene_name: String,
    /// Snapshot of every entity at save time.
    #[serde(default)]
    pub entities: Vec<EntitySnapshot>,
}

/// Per-entity data captured at save time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    /// Raw EntityId value.
    pub id: u64,
    /// Display name from `ObjectMeta`.
    pub name: String,
    /// World-space position [x, y, z] in metres.
    pub position: [f32; 3],
}

impl FluidEnvelope {
    /// Creates a new envelope with the current app version.
    pub fn new(scene_name: impl Into<String>) -> Self {
        Self {
            format_version: 1,
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            scene_name: scene_name.into(),
            entities: Vec::new(),
        }
    }
}

// ── I/O ───────────────────────────────────────────────────────────────────────

/// Saves a [`FluidEnvelope`] to `path` as a MessagePack file (map-based, DEC-011).
///
/// Creates the parent directory if it does not exist.
///
/// # Errors
/// Returns an error if the directory cannot be created, the file cannot be
/// written, or MessagePack serialization fails.
pub fn save(path: &Path, envelope: &FluidEnvelope) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    // rmp_serde::to_vec_named → map-based encoding (DEC-011: field names as keys).
    let bytes = rmp_serde::to_vec_named(envelope)?;
    std::fs::write(path, bytes)?;
    Ok(())
}

/// Loads a [`FluidEnvelope`] from a MessagePack `.fluid` file at `path`.
///
/// # Errors
/// Returns an error if the file cannot be read or the MessagePack is malformed.
pub fn load(path: &Path) -> Result<FluidEnvelope, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(path)?;
    let envelope: FluidEnvelope = rmp_serde::from_slice(&bytes)?;
    Ok(envelope)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_empty_scene() {
        let env = FluidEnvelope::new("Test Scene");
        let bytes = rmp_serde::to_vec_named(&env).expect("serialize");
        let decoded: FluidEnvelope = rmp_serde::from_slice(&bytes).expect("deserialize");
        assert_eq!(decoded.scene_name, "Test Scene");
        assert_eq!(decoded.format_version, 1);
        assert!(decoded.entities.is_empty());
    }

    #[test]
    fn roundtrip_with_entity() {
        let mut env = FluidEnvelope::new("Scene");
        env.entities.push(EntitySnapshot {
            id: 42,
            name: "Cube".to_string(),
            position: [1.0, 2.0, 3.0],
        });
        let bytes = rmp_serde::to_vec_named(&env).expect("serialize");
        let decoded: FluidEnvelope = rmp_serde::from_slice(&bytes).expect("deserialize");
        assert_eq!(decoded.entities.len(), 1);
        assert_eq!(decoded.entities[0].position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn map_based_encoding_contains_field_names() {
        // DEC-011 compliance: field names must be present as string keys in the
        // serialized bytes (map encoding), not just positional indices (array encoding).
        let env = FluidEnvelope::new("FieldNameTest");
        let bytes = rmp_serde::to_vec_named(&env).expect("serialize");
        // "scene_name" must appear as a UTF-8 key in the binary payload.
        let haystack = String::from_utf8_lossy(&bytes);
        assert!(
            haystack.contains("scene_name"),
            "MessagePack payload missing field name key 'scene_name' — array encoding used instead of map encoding"
        );
    }
}
