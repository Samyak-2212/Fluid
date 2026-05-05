//! Component plugin interface — reads `config/component_manifest.toml` (DEC-007).
//!
//! Zero hardcoded component lists. The manifest drives the UI component picker
//! and subprocess tier selection.

use serde::Deserialize;

/// A single component plugin entry from `config/component_manifest.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct ComponentEntry {
    /// Matches crate name and feature flag exactly (snake_case).
    pub id: String,
    /// Display name shown in UI.
    pub label: String,
    /// Subprocess binary name (without extension).
    pub binary: String,
    /// Minimum capability tier required.
    pub min_tier: u32,
    /// Tooltip / help text.
    pub description: String,
    /// If false, hidden from UI and not launched.
    pub enabled: bool,
}

/// Loaded component manifest.
#[derive(Debug, Default, Deserialize)]
pub struct ComponentManifest {
    #[serde(default)]
    pub component: Vec<ComponentEntry>,
}

/// Loads the component manifest from `config/component_manifest.toml`.
///
/// Returns an empty manifest (no panic) if the file is missing or malformed.
pub fn load_manifest(config_dir: &std::path::Path) -> ComponentManifest {
    let path = config_dir.join("component_manifest.toml");
    match std::fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            log::warn!("component_manifest.toml parse error: {e}; using empty manifest");
            ComponentManifest::default()
        }),
        Err(e) => {
            log::warn!("component_manifest.toml not found at {}: {e}", path.display());
            ComponentManifest::default()
        }
    }
}
