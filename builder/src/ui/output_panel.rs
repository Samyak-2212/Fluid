// builder/src/ui/output_panel.rs
//
// Scrollable live output panel streaming cargo stdout/stderr.
// Auto-scrolls to bottom while building; user can scroll up freely.

use egui::Ui;

pub fn render_output_panel(ui: &mut Ui, lines: &[String], is_running: bool) {
    egui::Frame::dark_canvas(ui.style()).show(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("output_scroll")
            .auto_shrink([false, false])
            .stick_to_bottom(is_running)
            .show(ui, |ui| {
                for line in lines {
                    let color = if line.starts_with("error") {
                        egui::Color32::from_rgb(255, 80, 80)
                    } else if line.starts_with("warning") {
                        egui::Color32::from_rgb(255, 200, 50)
                    } else {
                        egui::Color32::LIGHT_GRAY
                    };
                    ui.label(
                        egui::RichText::new(line)
                            .monospace()
                            .color(color),
                    );
                }
            });
    });
}
