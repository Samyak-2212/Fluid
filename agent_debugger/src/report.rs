//! Report module — JSON diff report (before/after).
//!
//! Produces `report.json` for each session, committed to git.
//! Compares state_before and state_after snapshots and lists widget changes.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::DebugError;
use crate::state::{AppStateSnapshot, WidgetEntry};

/// A diff entry describing a changed field.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldDiff {
    pub field: String,
    pub before: String,
    pub after: String,
}

/// A diff entry for a widget change between tree snapshots.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WidgetDiff {
    pub widget_id: String,
    pub kind: String,
    /// `"added"`, `"removed"`, or `"changed"`.
    pub change: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<FieldDiff>,
}

/// The full session report, written to `report.json`.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionReport {
    pub session_id: String,
    pub timestamp: String,
    /// Action that was performed (from the control request).
    pub action: String,
    /// True if screenshot was unavailable.
    pub incomplete: bool,
    /// Diffs between state_before and state_after.
    pub state_diffs: Vec<FieldDiff>,
    /// Diffs between tree_before and tree_after.
    pub widget_diffs: Vec<WidgetDiff>,
    /// Free-form notes (e.g. "screenshot_not_implemented").
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

impl SessionReport {
    /// Build a report by comparing before/after state and widget trees.
    pub fn build(
        session_id: &str,
        timestamp: &str,
        action: &str,
        incomplete: bool,
        state_before: &AppStateSnapshot,
        state_after: &AppStateSnapshot,
        widgets_before: &[WidgetEntry],
        widgets_after: &[WidgetEntry],
        notes: Vec<String>,
    ) -> Self {
        let state_diffs = diff_state(state_before, state_after);
        let widget_diffs = diff_widgets(widgets_before, widgets_after);

        SessionReport {
            session_id: session_id.to_string(),
            timestamp: timestamp.to_string(),
            action: action.to_string(),
            incomplete,
            state_diffs,
            widget_diffs,
            notes,
        }
    }

    /// Serialize to JSON and write to `path`.
    pub fn save(&self, path: &Path) -> Result<(), DebugError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DebugError::Io(format!("create_dir_all: {e}")))?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
            .map_err(|e| DebugError::Io(format!("write report to {}: {e}", path.display())))?;
        log::info!("Report saved: {}", path.display());
        Ok(())
    }
}

/// Diff two AppStateSnapshots, returning a list of changed fields.
fn diff_state(before: &AppStateSnapshot, after: &AppStateSnapshot) -> Vec<FieldDiff> {
    let mut diffs = Vec::new();

    macro_rules! diff_field {
        ($field:ident) => {
            if before.$field != after.$field {
                diffs.push(FieldDiff {
                    field: stringify!($field).to_string(),
                    before: format!("{}", before.$field),
                    after: format!("{}", after.$field),
                });
            }
        };
    }

    diff_field!(frame);
    diff_field!(sim_time);
    diff_field!(sim_status);
    diff_field!(scene_name);
    diff_field!(entity_count);
    diff_field!(headless);
    diff_field!(tier);
    diff_field!(fps);

    diffs
}

/// Diff two widget lists (before/after tree), returning added/removed/changed entries.
fn diff_widgets(before: &[WidgetEntry], after: &[WidgetEntry]) -> Vec<WidgetDiff> {
    let mut diffs = Vec::new();

    // Find removed or changed widgets.
    for bw in before {
        match after.iter().find(|aw| aw.id == bw.id) {
            None => {
                diffs.push(WidgetDiff {
                    widget_id: bw.id.clone(),
                    kind: bw.kind.clone(),
                    change: "removed".to_string(),
                    fields: vec![],
                });
            }
            Some(aw) => {
                let mut fields = Vec::new();
                if bw.enabled != aw.enabled {
                    fields.push(FieldDiff {
                        field: "enabled".to_string(),
                        before: bw.enabled.to_string(),
                        after: aw.enabled.to_string(),
                    });
                }
                if bw.value != aw.value {
                    fields.push(FieldDiff {
                        field: "value".to_string(),
                        before: format!("{:?}", bw.value),
                        after: format!("{:?}", aw.value),
                    });
                }
                if bw.label != aw.label {
                    fields.push(FieldDiff {
                        field: "label".to_string(),
                        before: bw.label.clone(),
                        after: aw.label.clone(),
                    });
                }
                if !fields.is_empty() {
                    diffs.push(WidgetDiff {
                        widget_id: bw.id.clone(),
                        kind: bw.kind.clone(),
                        change: "changed".to_string(),
                        fields,
                    });
                }
            }
        }
    }

    // Find added widgets.
    for aw in after {
        if before.iter().all(|bw| bw.id != aw.id) {
            diffs.push(WidgetDiff {
                widget_id: aw.id.clone(),
                kind: aw.kind.clone(),
                change: "added".to_string(),
                fields: vec![],
            });
        }
    }

    diffs
}
