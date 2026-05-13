//! C8-Assets — material/simulation preset loader.
//!
//! Presets are TOML files bundled in `app/assets/presets/`.
//! They are loaded at runtime from disk relative to the executable, or from
//! the source tree during development (via `CARGO_MANIFEST_DIR` if present).
//!
//! # Session 7 scope
//! Three built-in presets are bundled: water, air, steel.
//! Future sessions may add a preset browser UI and user-defined presets stored
//! in the OS config directory (DEC-016).

use serde::Deserialize;
use std::path::{Path, PathBuf};

// ── MaterialPreset ────────────────────────────────────────────────────────────

/// Shared fields that every material preset TOML must contain.
///
/// Format-specific fields (e.g. `bulk_modulus`, `youngs_modulus`) are stored in
/// the raw `extras` map for forward compatibility and are not yet exposed in the
/// UI. All numeric fields use SI units.
#[derive(Debug, Clone, Deserialize)]
pub struct MaterialPreset {
    /// Display name shown in the Simulation menu and Properties panel.
    pub name: String,
    /// Material class: "liquid", "gas", or "solid".
    pub material: String,
    /// Density in kg/m³.
    pub density: f64,
    /// Dynamic viscosity in Pa·s. Not present for solid presets.
    #[serde(default)]
    pub viscosity: f64,
}

// ── Preset database ───────────────────────────────────────────────────────────

/// All built-in presets, loaded once at startup.
#[derive(Debug, Default)]
pub struct PresetDb {
    pub presets: Vec<MaterialPreset>,
}

impl PresetDb {
    /// Loads all `*.toml` files from the preset directory.
    ///
    /// Uses `CARGO_MANIFEST_DIR` (set by Cargo during `cargo run`) to locate
    /// `app/assets/presets/` during development. In a distribution build, falls
    /// back to `<exe_dir>/assets/presets/`.
    pub fn load() -> Self {
        let dir = Self::find_preset_dir();
        let mut db = Self::default();

        let dir = match dir {
            Some(d) => d,
            None => {
                log::warn!("PresetDb::load: preset directory not found; using empty database");
                return db;
            }
        };

        match std::fs::read_dir(&dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                        match Self::load_one(&path) {
                            Ok(preset) => {
                                log::debug!("Loaded preset \"{}\" from {:?}", preset.name, path);
                                db.presets.push(preset);
                            }
                            Err(e) => {
                                log::warn!("Failed to parse preset {:?}: {e}", path);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Cannot read preset dir {:?}: {e}", dir);
            }
        }

        db
    }

    /// Returns the preset with the given name (case-sensitive), if found.
    pub fn get(&self, name: &str) -> Option<&MaterialPreset> {
        self.presets.iter().find(|p| p.name == name)
    }

    /// Returns names of all loaded presets, in load order.
    pub fn names(&self) -> Vec<&str> {
        self.presets.iter().map(|p| p.name.as_str()).collect()
    }

    fn load_one(path: &Path) -> Result<MaterialPreset, Box<dyn std::error::Error>> {
        let text = std::fs::read_to_string(path)?;
        let preset: MaterialPreset = toml::from_str(&text)?;
        Ok(preset)
    }

    fn find_preset_dir() -> Option<PathBuf> {
        // Development: Cargo sets CARGO_MANIFEST_DIR to the `app/` crate root.
        if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
            let dev_path = PathBuf::from(manifest).join("assets").join("presets");
            if dev_path.is_dir() {
                return Some(dev_path);
            }
        }
        // Distribution: assets/ next to the running binary.
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                let dist_path = exe_dir.join("assets").join("presets");
                if dist_path.is_dir() {
                    return Some(dist_path);
                }
            }
        }
        None
    }
}
