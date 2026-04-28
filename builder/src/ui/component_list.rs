// builder/src/ui/component_list.rs
//
// Component selection checkboxes.
// Greyed out if a required dependency component is deselected.
// Warns on dependency violations without silently enabling components.

use egui::Ui;
use std::collections::HashMap;

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
/// Returns a warning string if any dependency constraint is violated.
pub fn render_component_list(
    ui: &mut Ui,
    components: &mut Vec<ComponentEntry>,
) -> Option<String> {
    let mut warning: Option<String> = None;

    // Build a quick lookup of current selections.
    let selections: HashMap<String, bool> = components
        .iter()
        .map(|c| (c.name.clone(), c.selected))
        .collect();

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
