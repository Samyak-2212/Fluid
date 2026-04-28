// builder/src/ui/flag_panel.rs
//
// Renders flag widgets dynamically from the loaded BuilderConfig.
// Supports: bool (checkbox), select (ComboBox), string (TextEdit).

use egui::Ui;
use crate::config::FlagState;

/// Render all flag widgets into the given UI area.
/// Mutates each FlagState's current_value in place.
pub fn render_flag_panel(ui: &mut Ui, flags: &mut Vec<FlagState>) {
    egui::ScrollArea::vertical()
        .id_salt("flag_scroll")
        .show(ui, |ui| {
            for flag in flags.iter_mut() {
                ui.horizontal(|ui| {
                    match flag.entry.flag_type.as_str() {
                        "bool" => {
                            let mut checked = flag.current_value == "true";
                            if ui.checkbox(&mut checked, &flag.entry.label).changed() {
                                flag.current_value = if checked { "true".to_string() } else { "false".to_string() };
                            }
                        }
                        "select" => {
                            ui.label(&flag.entry.label);
                            egui::ComboBox::from_id_salt(&flag.entry.name)
                                .selected_text(&flag.current_value)
                                .show_ui(ui, |ui| {
                                    for opt in &flag.entry.options {
                                        ui.selectable_value(
                                            &mut flag.current_value,
                                            opt.clone(),
                                            opt,
                                        );
                                    }
                                });
                        }
                        "string" | _ => {
                            ui.label(&flag.entry.label);
                            ui.text_edit_singleline(&mut flag.current_value);
                        }
                    }
                });
                // Tooltip on hover for the last rendered widget.
                ui.label(
                    egui::RichText::new(&flag.entry.description)
                        .small()
                        .color(egui::Color32::GRAY),
                );
                ui.separator();
            }
        });
}
