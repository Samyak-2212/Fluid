//! State and tree fetch — GET /state and GET /tree.
//!
//! Both endpoints return JSON. Widget IDs from /tree are used by control.rs
//! at runtime (DEC-C9-006 — never hardcoded).

use serde::{Deserialize, Serialize};

use crate::config::AgentDebuggerConfig;
use crate::error::DebugError;
use crate::health::authed_get;

// ── /state ──────────────────────────────────────────────────────────────────

/// Full snapshot returned by `GET /state`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppStateSnapshot {
    pub frame: u64,
    pub sim_time: f64,
    pub sim_status: String,
    pub scene_name: String,
    pub entity_count: u64,
    pub headless: bool,
    pub tier: u32,
    pub fps: f64,
}

/// Fetch the current application state.
pub fn fetch_state(cfg: &AgentDebuggerConfig) -> Result<AppStateSnapshot, DebugError> {
    let url = format!("{}/state", cfg.base_url());
    let resp = authed_get(cfg, &url)?;
    let snap: AppStateSnapshot = resp
        .into_json()
        .map_err(|e| DebugError::Json(e.to_string()))?;
    log::debug!("State fetched: frame={} sim_status={}", snap.frame, snap.sim_status);
    Ok(snap)
}

// ── /tree ───────────────────────────────────────────────────────────────────

/// A single widget entry in the C8 widget registry.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WidgetEntry {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub enabled: bool,
    pub value: Option<String>,
}

/// Full response from `GET /tree`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TreeResponse {
    pub headless: bool,
    pub widgets: Vec<WidgetEntry>,
}

/// Fetch the widget registry.
pub fn fetch_tree(cfg: &AgentDebuggerConfig) -> Result<TreeResponse, DebugError> {
    let url = format!("{}/tree", cfg.base_url());
    let resp = authed_get(cfg, &url)?;
    let tree: TreeResponse = resp
        .into_json()
        .map_err(|e| DebugError::Json(e.to_string()))?;
    log::debug!(
        "Tree fetched: headless={}, widget_count={}",
        tree.headless,
        tree.widgets.len()
    );
    Ok(tree)
}

/// Find a widget by its id string.
#[allow(dead_code)]
pub fn find_widget<'a>(tree: &'a TreeResponse, id: &str) -> Option<&'a WidgetEntry> {
    tree.widgets.iter().find(|w| w.id == id)
}

/// Find a widget by label (case-insensitive).
#[allow(dead_code)]
pub fn find_widget_by_label<'a>(tree: &'a TreeResponse, label: &str) -> Option<&'a WidgetEntry> {
    let label_lower = label.to_lowercase();
    tree.widgets
        .iter()
        .find(|w| w.label.to_lowercase() == label_lower)
}

// ── /logs ───────────────────────────────────────────────────────────────────

/// Response from `GET /logs`.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogsResponse {
    pub lines: Vec<String>,
}

/// Fetch the log tail.
pub fn fetch_logs(cfg: &AgentDebuggerConfig) -> Result<LogsResponse, DebugError> {
    let url = format!("{}/logs", cfg.base_url());
    let resp = authed_get(cfg, &url)?;
    let logs: LogsResponse = resp
        .into_json()
        .map_err(|e| DebugError::Json(e.to_string()))?;
    Ok(logs)
}
