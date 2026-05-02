//! Cleanup module — `cleanup --session <ts>` and `cleanup --committed` subcommands.
//!
//! Session archival policy (DEC-C9-007):
//!   - JSON files: committed to git, kept permanently.
//!   - PNG files: gitignored, deleted by this command after git commit.
//!
//! Usage:
//!   cargo run -p agent_debugger -- cleanup --session <timestamp>
//!   cargo run -p agent_debugger -- cleanup --committed

use std::path::{Path, PathBuf};

use crate::error::DebugError;

/// Delete PNG files in `agent_debugger/sessions/<session_id>/`.
///
/// Returns the list of deleted paths.
pub fn cleanup_session(sessions_dir: &Path, session_id: &str) -> Result<Vec<PathBuf>, DebugError> {
    let session_dir = sessions_dir.join(session_id);

    if !session_dir.exists() {
        return Err(DebugError::Io(format!(
            "session directory not found: {}",
            session_dir.display()
        )));
    }

    delete_pngs_in(&session_dir)
}

/// Delete PNG files across ALL sessions in `agent_debugger/sessions/`.
///
/// Intended for use after `git commit` of all JSON archives.
/// Returns the list of deleted paths.
pub fn cleanup_all_committed(sessions_dir: &Path) -> Result<Vec<PathBuf>, DebugError> {
    if !sessions_dir.exists() {
        // Nothing to clean.
        return Ok(vec![]);
    }

    let entries = std::fs::read_dir(sessions_dir)
        .map_err(|e| DebugError::Io(format!("read_dir {}: {e}", sessions_dir.display())))?;

    let mut deleted = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| DebugError::Io(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            let mut batch = delete_pngs_in(&path)?;
            deleted.append(&mut batch);
        }
    }
    Ok(deleted)
}

/// Delete all `*.png` files in `dir`. Returns list of deleted paths.
fn delete_pngs_in(dir: &Path) -> Result<Vec<PathBuf>, DebugError> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| DebugError::Io(format!("read_dir {}: {e}", dir.display())))?;

    let mut deleted = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| DebugError::Io(e.to_string()))?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext.eq_ignore_ascii_case("png") {
                    std::fs::remove_file(&path)
                        .map_err(|e| DebugError::Io(format!("remove {}: {e}", path.display())))?;
                    log::info!("Deleted: {}", path.display());
                    deleted.push(path);
                }
            }
        }
    }
    Ok(deleted)
}
