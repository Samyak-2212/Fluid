//! User preferences — OS config dir via `directories`, hot-swap via `notify` + `arc-swap`.
//!
//! DEC-016: preferences stored in OS-standard config dir (NOT `config/`).
//!   - Windows:   %APPDATA%\Fluid\prefs.toml
//!   - Linux:     ~/.config/fluid/prefs.toml
//!   - macOS:     ~/Library/Application Support/Fluid/prefs.toml
//!
//! DEC-017: File watcher MUST use `iced::Subscription` + `stream::channel`.
//!   NOT bare threads. The `notify` watcher is integrated via Subscription::run.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use arc_swap::ArcSwap;
use std::sync::Arc;

/// User preferences (stored in OS config dir, DEC-016).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPrefs {
    /// Autosave interval in seconds. 0 = disabled.
    pub autosave_interval_secs: u64,
    /// Active capability tier override. None = auto-detect.
    pub tier_override: Option<u32>,
    /// Theme name ("dark" | TOML file path for custom theme).
    pub theme: String,
    /// UI language code (e.g. "en"). Reserved for future localisation.
    pub language: String,
    /// Most recently opened file paths (newest first, max 10).
    pub recent_files: Vec<String>,
}

impl Default for UserPrefs {
    fn default() -> Self {
        Self {
            autosave_interval_secs: 300,
            tier_override: None,
            theme: "dark".to_string(),
            language: "en".to_string(),
            recent_files: Vec::new(),
        }
    }
}

/// Returns the OS-standard path for `prefs.toml` (DEC-016).
///
/// Creates the directory if it does not exist.
pub fn prefs_path() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "Fluid")?;
    let dir = dirs.config_dir();
    std::fs::create_dir_all(dir).ok()?;
    Some(dir.join("prefs.toml"))
}

/// Loads user preferences from the OS config dir.
///
/// Returns `UserPrefs::default()` on any error — no panic on missing or malformed file.
pub fn load_prefs() -> UserPrefs {
    let path = match prefs_path() {
        Some(p) => p,
        None => return UserPrefs::default(),
    };
    match std::fs::read_to_string(&path) {
        Ok(content) => toml::from_str(&content).unwrap_or_else(|e| {
            log::warn!("prefs.toml parse error: {e}; using defaults");
            UserPrefs::default()
        }),
        Err(_) => UserPrefs::default(),
    }
}

/// Saves user preferences to the OS config dir.
pub fn save_prefs(prefs: &UserPrefs) -> Result<(), Box<dyn std::error::Error>> {
    let path = prefs_path().ok_or("could not determine config dir")?;
    let content = toml::to_string_pretty(prefs)?;
    std::fs::write(&path, content)?;
    Ok(())
}

/// Shared user preferences handle for hot-swap reads.
///
/// The main loop writes a new `Arc<UserPrefs>` when the file watcher detects a change.
/// All readers (theme, autosave, tier selection) read via `ArcSwap::load()` — zero locks.
pub type SharedPrefs = Arc<ArcSwap<UserPrefs>>;

/// Creates a new `SharedPrefs` loaded from the OS config dir.
pub fn shared_prefs() -> SharedPrefs {
    Arc::new(ArcSwap::new(Arc::new(load_prefs())))
}
