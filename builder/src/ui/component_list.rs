// builder/src/ui/component_list.rs
//
// Component selection checkboxes.
// Greyed out if a required dependency component is deselected.
// Warns on dependency violations without silently enabling components.

use egui::Ui;
use std::collections::HashMap;
use std::time::Duration;

use crate::state::ComponentStatus;

/// Format a Duration as "3.2s" (sub-minute) or "1m 04s" (60 s and above).
fn format_elapsed(d: Duration) -> String {
    let total_secs = d.as_secs();
    if total_secs < 60 {
        format!("{:.1}s", d.as_secs_f32())
    } else {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}m {:02}s", mins, secs)
    }
}

/// A selectable component entry.
#[derive(Debug, Clone)]
pub struct ComponentEntry {
    /// Feature name, e.g. "fluid_simulator"
    pub name: String,
    /// Human-readable label
    pub label: String,
    /// Feature names this component requires to be enabled.
    pub requires: Vec<String>,
    pub selected: bool,
}

/// Render the component list.
/// Mutates selection state in place.
/// `statuses` is the per-component build status map from `BuildState`; used
/// only for reading elapsed time — no mutations.
/// `flag_selections` maps flag names to their current bool value (from the
/// Build Flags panel); used so dependency checks span both panels.
/// Returns a warning string if any dependency constraint is violated.
pub fn render_component_list(
    ui: &mut Ui,
    components: &mut Vec<ComponentEntry>,
    statuses: &HashMap<String, ComponentStatus>,
    flag_selections: &HashMap<String, bool>,
) -> Option<String> {
    let mut warning: Option<String> = None;

    // Build a quick lookup of current selections — merge right-panel component
    // selections with left-panel flag selections so dependency checks span both.
    let mut selections: HashMap<String, bool> = flag_selections.clone();
    for c in components.iter() {
        if c.selected {
            selections.insert(c.name.clone(), true);
        }
    }

    for comp in components.iter_mut() {
        // Determine if any required dependency is deselected.
        let blocked_by: Vec<&str> = comp
            .requires
            .iter()
            .filter(|req| !selections.get(*req).copied().unwrap_or(false))
            .map(|s| s.as_str())
            .collect();

        let greyed = !blocked_by.is_empty();

        ui.add_enabled_ui(!greyed, |ui| {
            ui.horizontal(|ui| {
                if ui.checkbox(&mut comp.selected, &comp.label).changed() {
                    // If user just enabled this and dependencies are missing, revert and warn.
                    if comp.selected && greyed {
                        comp.selected = false;
                        warning = Some(format!(
                            "'{}' requires: {}",
                            comp.label,
                            blocked_by.join(", ")
                        ));
                    }
                }

                // Display elapsed time if available for this component.
                if let Some(status) = statuses.get(&comp.name) {
                    if let Some(elapsed) = status.elapsed() {
                        let (text, color) = match status {
                            ComponentStatus::Succeeded { .. } => (
                                format_elapsed(elapsed),
                                egui::Color32::from_rgb(100, 210, 100),
                            ),
                            ComponentStatus::Failed { .. } => (
                                format_elapsed(elapsed),
                                egui::Color32::from_rgb(255, 100, 100),
                            ),
                            _ => (
                                format_elapsed(elapsed),
                                egui::Color32::GRAY,
                            ),
                        };
                        ui.label(
                            egui::RichText::new(text)
                                .small()
                                .color(color),
                        );
                    }
                }
            });

            if greyed {
                ui.label(
                    egui::RichText::new(format!("Requires: {}", blocked_by.join(", ")))
                        .small()
                        .color(egui::Color32::DARK_GRAY),
                );
            }
        });
    }

    warning
}
