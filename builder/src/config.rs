// builder/src/config.rs
//
// Loads config/builder_flags.toml at startup and provides typed access to flag entries.
// No hardcoded values. Runtime panics on missing keys are forbidden — use defaults.

use serde::Deserialize;
use std::path::Path;

/// A single flag entry from builder_flags.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct FlagEntry {
    /// Machine name — env var name, cargo flag, or feature name.
    pub name: String,
    /// Kind: "env" | "cargo_flag" | "feature"
    pub kind: String,
    /// Human-readable label shown in UI.
    pub label: String,
    /// Tooltip / description shown on hover.
    pub description: String,
    /// Render type: "select" | "bool" | "string"
    #[serde(rename = "type")]
    pub flag_type: String,
    /// Allowed values for type = "select". Empty for other types.
    #[serde(default)]
    pub options: Vec<String>,
    /// Default value as a string.
    pub default: String,
}

/// Top-level container for the parsed TOML file.
#[derive(Debug, Clone, Deserialize)]
pub struct BuilderConfig {
    #[serde(rename = "flag")]
    pub flags: Vec<FlagEntry>,
}

impl BuilderConfig {
    /// Load and parse `builder_flags.toml` from the given path.
    /// Returns an error string on failure; never panics.
    pub fn load(path: &Path) -> Result<BuilderConfig, String> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {:?}: {}", path, e))?;
        let cfg: BuilderConfig =
            toml::from_str(&raw).map_err(|e| format!("parse error in {:?}: {}", path, e))?;
        Ok(cfg)
    }

    /// Returns a default (empty) config so the UI can still start even if the file is missing.
    #[allow(dead_code)]
    pub fn empty() -> BuilderConfig {
        BuilderConfig { flags: Vec::new() }
    }
}

/// Runtime state for a single flag: its current value as entered by the user.
#[derive(Debug, Clone)]
pub struct FlagState {
    pub entry: FlagEntry,
    pub current_value: String,
}

impl FlagState {
    pub fn from_entry(entry: FlagEntry) -> Self {
        let current_value = entry.default.clone();
        Self { entry, current_value }
    }
}
