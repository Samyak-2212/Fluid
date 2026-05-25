//! Asset import pipeline — glTF/OBJ/STL/FBX (DEC-003).
//!
//! glTF/GLB: implemented via the `gltf` crate.
//! OBJ:      implemented via `tobj`.
//! STL:      implemented via `stl_io`.
//! FBX:      stub — maturity concerns (see DEC-003 and research_dump/_INDEX.md).
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
/// extracted. Rotation, scale, and actual vertex data are C8-Viewport follow-up.
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

// ── OBJ import ────────────────────────────────────────────────────────────────

/// Loads a Wavefront OBJ file and returns one [`ImportedMesh`] per model.
///
/// Uses `tobj` with `GPU_LOAD_OPTIONS` (triangulates and deduplicates vertices).
/// OBJ has no node transform; all meshes originate at `[0.0, 0.0, 0.0]`.
/// The app layer spreads co-located meshes along X so they are individually
/// visible in the 3D viewport.
pub fn import_obj(path: &Path) -> Result<Vec<ImportedMesh>, Box<dyn std::error::Error>> {
    let (models, _materials) = tobj::load_obj(
        path,
        &tobj::GPU_LOAD_OPTIONS,
    )?;

    let mut meshes = Vec::new();
    for (index, model) in models.iter().enumerate() {
        let name = if model.name.is_empty() {
            format!("OBJ_Mesh_{index}")
        } else {
            model.name.clone()
        };
        // OBJ files have no node-level transforms; place at origin.
        // The ImportFile handler spreads these if they all land at origin.
        meshes.push(ImportedMesh { name, translation: [0.0, 0.0, 0.0] });
    }

    if meshes.is_empty() {
        log::warn!("import_obj: {:?} contained no models", path);
    }

    Ok(meshes)
}

// ── STL import ────────────────────────────────────────────────────────────────

/// Loads an STL file (ASCII or binary) and returns a single [`ImportedMesh`].
///
/// STL has no node hierarchy, no named objects, and no transforms.
/// A single mesh named after the file stem is returned at the origin.
pub fn import_stl(path: &Path) -> Result<Vec<ImportedMesh>, Box<dyn std::error::Error>> {
    let mut file = std::fs::OpenOptions::new().read(true).open(path)?;
    let stl = stl_io::read_stl(&mut file)?;

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("STL_Mesh")
        .to_owned();

    let triangle_count = stl.faces.len();
    if triangle_count == 0 {
        log::warn!("import_stl: {:?} contained no triangles", path);
        return Ok(vec![]);
    }

    log::info!("import_stl: {:?} — {} triangles", path, triangle_count);
    Ok(vec![ImportedMesh { name, translation: [0.0, 0.0, 0.0] }])
}

// ── Dispatch entry-point ─────────────────────────────────────────────────────

/// Imports a file, dispatching to the per-format implementation.
///
/// FBX returns a stub error until C8-Import follow-up sessions.
pub fn import_file(path: &Path) -> Result<Vec<ImportedMesh>, Box<dyn std::error::Error>> {
    match ImportFormat::from_path(path) {
        Some(ImportFormat::GlTF) => import_gltf(path),
        Some(ImportFormat::Obj)  => import_obj(path),
        Some(ImportFormat::Stl)  => import_stl(path),
        Some(ImportFormat::Fbx)  => Err("FBX import not yet implemented (fbxcel-dom maturity concerns — see DEC-003)".into()),
        None => Err(format!("unrecognised file extension: {:?}", path.extension()).into()),
    }
}
