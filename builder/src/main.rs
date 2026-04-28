// builder/src/main.rs
//
// Fluid Builder — egui-based native build UI.
// Loads all configuration from config/builder_flags.toml at startup.
// Invokes cargo as a background subprocess with non-blocking output streaming.
// Termination uses child.kill() — platform-safe on Windows and Unix.

mod config;
mod state;
mod subprocess;
mod ui;

use config::{BuilderConfig, FlagState};
use state::{BuildSessionState, BuildState};
use subprocess::{BuildProcess, OutputLine};
use ui::component_list::{ComponentEntry, render_component_list};
use ui::flag_panel::render_flag_panel;
use ui::output_panel::render_output_panel;

use eframe::egui;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

/// Locate config/builder_flags.toml relative to the workspace root.
/// Walks up from the executable path until finding a directory containing Cargo.toml.
fn locate_config() -> PathBuf {
    // Try workspace root relative to current working directory first.
    let cwd = std::env::current_dir().unwrap_or_default();
    let candidate = cwd.join("config").join("builder_flags.toml");
    if candidate.exists() {
        return candidate;
    }
    // Fallback: relative to the executable location.
    let mut dir = std::env::current_exe().unwrap_or_default();
    dir.pop(); // remove binary name
    for _ in 0..6 {
        let c = dir.join("config").join("builder_flags.toml");
        if c.exists() {
            return c;
        }
        dir.pop();
    }
    // Last resort — let load() fail with a useful message.
    cwd.join("config").join("builder_flags.toml")
}

/// Build the default component list.
/// In a real deployment the metadata would be read from each crate's Cargo.toml.
fn default_components() -> Vec<ComponentEntry> {
    vec![
        ComponentEntry {
            name: "fluid_simulator".into(),
            label: "Fluid Simulator".into(),
            requires: vec![],
            selected: false,
        },
        ComponentEntry {
            name: "aerodynamic_simulator".into(),
            label: "Aerodynamic Simulator".into(),
            requires: vec![],
            selected: false,
        },
        ComponentEntry {
            name: "motion_force_simulator".into(),
            label: "Motion & Force Simulator".into(),
            requires: vec![],
            selected: false,
        },
        ComponentEntry {
            name: "thermodynamic_simulator".into(),
            label: "Thermodynamic Simulator".into(),
            requires: vec![],
            selected: false,
        },
        ComponentEntry {
            name: "fem_structural".into(),
            label: "FEM Structural".into(),
            requires: vec![],
            selected: false,
        },
    ]
}

/// Main application struct — owns all UI + build state.
struct FluidBuilderApp {
    flags: Vec<FlagState>,
    components: Vec<ComponentEntry>,
    build_state: BuildState,
    process: Option<BuildProcess>,
    component_warning: Option<String>,
    config_error: Option<String>,
    release_warning_shown: bool,
    last_build_start: Option<Instant>,
}

impl FluidBuilderApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let config_path = locate_config();
        let (flags, config_error) = match BuilderConfig::load(&config_path) {
            Ok(cfg) => {
                let states: Vec<FlagState> =
                    cfg.flags.into_iter().map(FlagState::from_entry).collect();
                (states, None)
            }
            Err(e) => {
                eprintln!("Warning: failed to load builder_flags.toml: {}", e);
                (Vec::new(), Some(e))
            }
        };

        FluidBuilderApp {
            flags,
            components: default_components(),
            build_state: BuildState::new(),
            process: None,
            component_warning: None,
            config_error,
            release_warning_shown: false,
            last_build_start: None,
        }
    }

    /// Build and return the cargo Command according to current flag state.
    fn build_command(&self) -> Command {
        // Collect env flags, cargo flags, and feature flags.
        let mut cmd = Command::new("cargo");
        cmd.arg("build");

        let mut features: Vec<String> = Vec::new();
        let mut is_release = false;
        let mut tier = "0".to_string();

        for flag in &self.flags {
            match flag.entry.kind.as_str() {
                "env" => {
                    if flag.entry.name == "FLUID_TIER" {
                        tier = flag.current_value.clone();
                    }
                    cmd.env(&flag.entry.name, &flag.current_value);
                }
                "cargo_flag" => {
                    if flag.entry.name == "release" && flag.current_value == "true" {
                        cmd.arg("--release");
                        is_release = true;
                    } else if flag.current_value == "true" {
                        cmd.arg(format!("--{}", flag.entry.name));
                    }
                }
                "feature" => {
                    if flag.current_value == "true" {
                        features.push(flag.entry.name.clone());
                    }
                }
                _ => {}
            }
        }

        // Add component features that are selected.
        for comp in &self.components {
            if comp.selected {
                features.push(comp.name.clone());
            }
        }

        if !features.is_empty() {
            cmd.arg("--features");
            cmd.arg(features.join(","));
        }

        // Ensure FLUID_TIER is set (default 0 for debug, 2 for release).
        let effective_tier = if is_release && tier == "0" { "2" } else { &tier };
        cmd.env("FLUID_TIER", effective_tier);

        cmd
    }

    /// Start the build subprocess.
    fn start_build(&mut self) {
        self.build_state.reset();
        self.build_state.session = BuildSessionState::Running;
        self.last_build_start = Some(Instant::now());

        let mut cmd = self.build_command();

        match BuildProcess::spawn(&mut cmd) {
            Ok(proc) => {
                self.process = Some(proc);
            }
            Err(e) => {
                self.build_state
                    .push_output(format!("Failed to spawn cargo: {}", e));
                self.build_state.session = BuildSessionState::Finished;
            }
        }
    }

    /// Cancel the running build.
    fn cancel_build(&mut self) {
        if let Some(proc) = &mut self.process {
            if let Err(e) = proc.kill() {
                eprintln!("kill() failed: {}", e);
            }
        }
        self.process = None;
        self.build_state.session = BuildSessionState::Cancelled;
    }

    /// Poll the subprocess for new output and update state.
    fn poll_process(&mut self) {
        if self.build_state.session != BuildSessionState::Running {
            return;
        }

        if let Some(proc) = &mut self.process {
            let lines = proc.poll_output();
            for line in lines {
                match line {
                    OutputLine::Stdout(l) | OutputLine::Stderr(l) => {
                        self.build_state.push_output(l);
                    }
                }
            }
            if !proc.is_running() {
                let exit = proc.exit_status();
                self.build_state.session = BuildSessionState::Finished;
                let summary = match exit {
                    Some(s) if s.success() => "Build succeeded.".to_string(),
                    Some(s) => format!("Build failed (exit: {:?}).", s.code()),
                    None => "Build terminated.".to_string(),
                };
                self.build_state.push_output(summary);
                self.process = None;
            }
        }
    }

    /// Returns the current "release" flag state.
    fn is_release(&self) -> bool {
        self.flags
            .iter()
            .find(|f| f.entry.name == "release")
            .map(|f| f.current_value == "true")
            .unwrap_or(false)
    }
}

impl eframe::App for FluidBuilderApp {
    /// Called before each call to `ui`, and also when the UI is hidden but
    /// `request_repaint` was called. Do NOT show any UI here.
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_process();
        // Keep repainting while a build is running so output streams live.
        if self.build_state.session == BuildSessionState::Running {
            ctx.request_repaint();
        }
    }

    /// Required by eframe 0.34.1. All UI rendering goes here.
    /// Panels must be shown via `show_inside(ui, ...)` from within `fn ui`.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let is_running = self.build_state.session == BuildSessionState::Running;

        egui::Panel::top("title_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Fluid Builder");
                ui.separator();
                if let Some(e) = &self.config_error {
                    ui.label(
                        egui::RichText::new(format!("Config error: {}", e))
                            .color(egui::Color32::from_rgb(255, 80, 80)),
                    );
                }
            });
        });

        egui::Panel::bottom("control_bar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if is_running {
                    if ui
                        .button(egui::RichText::new("Cancel").color(egui::Color32::from_rgb(255, 80, 80)))
                        .clicked()
                    {
                        self.cancel_build();
                    }
                } else {
                    let build_label = if self.is_release() {
                        "Build (Release)"
                    } else {
                        "Build (Debug)"
                    };
                    if ui.button(build_label).clicked() {
                        if self.is_release() && !self.release_warning_shown {
                            self.release_warning_shown = true;
                            self.build_state.push_output(
                                "Warning: Release builds are slow. Do not use for iteration."
                                    .to_string(),
                            );
                        }
                        self.start_build();
                    }
                }

                ui.separator();

                let status_text = match &self.build_state.session {
                    BuildSessionState::Idle => "Idle".to_string(),
                    BuildSessionState::Running => {
                        if let Some(start) = self.last_build_start {
                            format!("Building… {:.1}s", start.elapsed().as_secs_f32())
                        } else {
                            "Building…".to_string()
                        }
                    }
                    BuildSessionState::Finished => "Finished".to_string(),
                    BuildSessionState::Cancelled => "Cancelled".to_string(),
                };
                ui.label(status_text);
            });
        });

        egui::Panel::left("flags_panel")
            .resizable(true)
            .min_size(260.0)
            .show_inside(ui, |ui| {
                ui.heading("Build Flags");
                ui.separator();
                render_flag_panel(ui, &mut self.flags);
            });

        egui::Panel::right("components_panel")
            .resizable(true)
            .min_size(200.0)
            .show_inside(ui, |ui| {
                ui.heading("Components");
                ui.separator();
                let warn = render_component_list(ui, &mut self.components);
                if let Some(w) = warn {
                    self.component_warning = Some(w);
                }
                if let Some(w) = &self.component_warning {
                    ui.separator();
                    ui.label(
                        egui::RichText::new(w)
                            .color(egui::Color32::from_rgb(255, 200, 50))
                            .small(),
                    );
                    if ui.button("Dismiss").clicked() {
                        self.component_warning = None;
                    }
                }
            });

        // Remaining space goes to the output panel.
        ui.vertical(|ui| {
            ui.heading("Output");
            ui.separator();
            render_output_panel(ui, &self.build_state.output_lines, is_running);
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Fluid Builder")
            .with_inner_size([1100.0, 700.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Fluid Builder",
        native_options,
        Box::new(|cc| Ok(Box::new(FluidBuilderApp::new(cc)))),
    )
}
