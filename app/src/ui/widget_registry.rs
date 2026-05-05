//! Widget registry — maps widget IDs to metadata for the C9 debug interface.
//!
//! # Design (DEC-013)
//! Iced 0.13 has no AccessKit support. C8 maintains its own widget registry
//! instead. Every interactive widget must call `.id(iced::widget::Id::new("unique_id"))`
//! and register itself here.
//!
//! The debug server holds an `Arc<RwLock<WidgetRegistry>>` clone. Writes happen
//! from the UI thread; reads happen from the debug server thread.

use std::collections::HashMap;
use serde::Serialize;

/// Metadata about a registered widget, visible to C9 via `GET /tree`.
#[derive(Debug, Clone, Serialize)]
pub struct WidgetEntry {
    /// The widget's `.id()` string — must be unique across the entire UI.
    pub id: String,
    /// Human-readable widget kind (e.g. "Button", "TextInput", "Slider").
    pub kind: String,
    /// Human-readable label shown to the user, if any.
    pub label: Option<String>,
    /// Whether the widget is currently enabled and interactive.
    pub enabled: bool,
    /// Current string value, if the widget holds text or a scalar.
    pub value: Option<String>,
}

impl WidgetEntry {
    pub fn button(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: "Button".to_string(),
            label: Some(label.into()),
            enabled: true,
            value: None,
        }
    }

    pub fn text_input(id: impl Into<String>, label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            kind: "TextInput".to_string(),
            label: Some(label.into()),
            enabled: true,
            value: Some(value.into()),
        }
    }

    pub fn slider(id: impl Into<String>, label: impl Into<String>, value: f64) -> Self {
        Self {
            id: id.into(),
            kind: "Slider".to_string(),
            label: Some(label.into()),
            enabled: true,
            value: Some(value.to_string()),
        }
    }
}

/// The widget registry.
///
/// Owned by C8-UI (at `app/src/ui/widget_registry.rs`).
/// The debug server holds a shared `Arc<RwLock<WidgetRegistry>>`.
/// Rebuild from scratch each UI frame — stale entries are overwritten.
#[derive(Debug, Default)]
pub struct WidgetRegistry {
    entries: HashMap<String, WidgetEntry>,
}

impl WidgetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers or updates a widget entry.
    /// Call this once per interactive widget per UI frame rebuild.
    pub fn register(&mut self, entry: WidgetEntry) {
        self.entries.insert(entry.id.clone(), entry);
    }

    /// Removes a widget entry (e.g. when a panel is closed).
    pub fn unregister(&mut self, id: &str) {
        self.entries.remove(id);
    }

    /// Clears all entries — call at the start of each frame rebuild.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns a snapshot of all current entries for the debug server.
    /// Returns a `Vec` sorted by widget ID for deterministic ordering.
    pub fn snapshot(&self) -> Vec<WidgetEntry> {
        let mut v: Vec<_> = self.entries.values().cloned().collect();
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    }

    /// Returns the entry for a given widget ID, if registered.
    pub fn get(&self, id: &str) -> Option<&WidgetEntry> {
        self.entries.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_retrieve() {
        let mut r = WidgetRegistry::new();
        r.register(WidgetEntry::button("btn_run", "Run Simulation"));
        let entry = r.get("btn_run").expect("entry should exist");
        assert_eq!(entry.kind, "Button");
        assert_eq!(entry.label.as_deref(), Some("Run Simulation"));
    }

    #[test]
    fn clear_removes_all() {
        let mut r = WidgetRegistry::new();
        r.register(WidgetEntry::button("a", "A"));
        r.register(WidgetEntry::button("b", "B"));
        r.clear();
        assert!(r.snapshot().is_empty());
    }

    #[test]
    fn snapshot_is_sorted() {
        let mut r = WidgetRegistry::new();
        r.register(WidgetEntry::button("z_btn", "Z"));
        r.register(WidgetEntry::button("a_btn", "A"));
        let snap = r.snapshot();
        assert_eq!(snap[0].id, "a_btn");
        assert_eq!(snap[1].id, "z_btn");
    }
}
