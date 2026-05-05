//! Asset import pipeline — glTF/OBJ/STL/FBX (DEC-003).
//!
//! Stub skeleton — full implementation is C8-Import sub-coordinator work.
//! STEP (ISO 10303) deferred to v2 (DEC-014).

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
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_lowercase().as_str() {
            "gltf" | "glb" => Some(Self::GlTF),
            "obj"           => Some(Self::Obj),
            "stl"           => Some(Self::Stl),
            "fbx"           => Some(Self::Fbx),
            _               => None,
        }
    }
}

/// Import a mesh file and return a handle to the imported data.
///
/// Stub — full implementation is C8-Import work.
pub fn import(_path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    // TODO(C8-Import): implement per-format import pipeline.
    Err("not_implemented".into())
}
