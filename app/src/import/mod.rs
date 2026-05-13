//! Asset import pipeline — glTF/OBJ/STL/FBX (DEC-003).
//!
//! glTF/GLB: implemented via the `gltf` crate.
//! OBJ/STL/FBX: stubs — C8-Import sub-coordinator follow-up.
//! STEP (ISO 10303) deferred to v2 (DEC-014).

use std::path::Path;

// ── ImportFormat ──────────────────────────────────────────────────────────────

/// Supported import formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportFormat {
    GlTF,
    Obj,
    Stl,
    /// [UNVERIFIED: fbxcel-dom maturity — see DEC-003 and research_dump/_INDEX.md]
    Fbx,
}

impl ImportFormat {
    /// Detects the format from the file extension.
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_lowercase().as_str() {
            "gltf" | "glb" => Some(Self::GlTF),
            "obj"           => Some(Self::Obj),
            "stl"           => Some(Self::Stl),
            "fbx"           => Some(Self::Fbx),
            _               => None,
        }
    }
}

// ── ImportedMesh ──────────────────────────────────────────────────────────────

/// A single imported mesh node with display name and world-space translation.
///
/// Callers spawn one scene entity per `ImportedMesh` and set its position from
/// `translation`. Full mesh geometry upload to wgpu is C8-Viewport follow-up.
#[derive(Debug, Clone)]
pub struct ImportedMesh {
    /// Node name from the file, or a generated fallback like "Mesh_0".
    pub name: String,
    /// World-space position extracted from the node's local transform, in metres.
    pub translation: [f32; 3],
}

// ── glTF import ───────────────────────────────────────────────────────────────

/// Loads a glTF 2.0 or GLB file and returns one [`ImportedMesh`] per mesh node.
///
/// Only the node name and translation component of the local transform are
/// extracted this session. Rotation, scale, and actual vertex data are
/// C8-Viewport follow-up.
pub fn import_gltf(path: &Path) -> Result<Vec<ImportedMesh>, Box<dyn std::error::Error>> {
    let (doc, _buffers, _images) = gltf::import(path)?;

    let mut meshes = Vec::new();
    for node in doc.nodes() {
        // Skip nodes without a mesh — cameras, lights, empties, etc.
        if node.mesh().is_none() {
            continue;
        }

        let name = node
            .name()
            .map(|n| n.to_owned())
            .unwrap_or_else(|| format!("Mesh_{}", node.index()));

        // Decompose the local transform into (translation, rotation, scale).
        let (t, _r, _s) = node.transform().decomposed();
        let translation = [t[0], t[1], t[2]];

        meshes.push(ImportedMesh { name, translation });
    }

    if meshes.is_empty() {
        // File loaded but had no mesh nodes — still a success; caller handles.
        log::warn!("import_gltf: {:?} contained no mesh nodes", path);
    }

    Ok(meshes)
}

// ── Dispatch entry-point ─────────────────────────────────────────────────────

/// Imports a file, dispatching to the per-format implementation.
///
/// OBJ/STL/FBX return a stub error until C8-Import follow-up sessions.
pub fn import_file(path: &Path) -> Result<Vec<ImportedMesh>, Box<dyn std::error::Error>> {
    match ImportFormat::from_path(path) {
        Some(ImportFormat::GlTF) => import_gltf(path),
        Some(fmt) => Err(format!("{fmt:?} import not yet implemented (C8-Import follow-up)").into()),
        None => Err(format!("unrecognised file extension: {:?}", path.extension()).into()),
    }
}

