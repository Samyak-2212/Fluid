//! `FluidApp` — the top-level Iced application struct.
//!
//! Implements the four methods required by `iced::application`:
//!   - `update(&mut self, msg: AppMessage) -> iced::Task<AppMessage>`
//!   - `view(&self) -> iced::Element<'_, AppMessage>`
//!   - `theme(&self) -> iced::Theme`
//!   - `subscription(&self) -> iced::Subscription<AppMessage>`
//!
//! # Session-3 scope
//! The viewport panel is a placeholder container — wgpu integration is
//! C8-Viewport work (session 4).  All other panels render lightweight
//! structural scaffolding (title bars, empty bodies).
//!
//! # Widget IDs (DEC-013)
//! Every interactive widget MUST call `.id(iced::widget::Id::new("…"))`.
//! Missing IDs are invisible to C9.  A lint warning is emitted in debug
//! builds for any interactive widget without an explicit id.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use iced::{
    Element, Subscription, Task, Theme,
    keyboard,
    widget::{
        button, column, container, horizontal_space, pane_grid,
        row, scrollable, text, text_input, Space,
        shader,
    },
    Color, Length,
};

use crate::sim_bridge::SimState;
use crate::assets::PresetDb;

use fluid_core::ecs::traits::EntityId;

use crate::{
    debug_server::AppStateSnapshot,
    file::{self, EntitySnapshot, FluidEnvelope},
    scene::{command::CommandHistory, Scene},
    ui::{
        layout::{build_pane_state, Panel},
        theme::fluid_theme,
        widget_registry::WidgetRegistry,
    },
    viewport::{ViewportState, ViewportProgram},
};

// ── MenuTarget ────────────────────────────────────────────────────────────────

/// Which top-level menu is currently open (None = all closed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuTarget {
    File,
    Edit,
    Simulation,
    View,
    Help,
}

// ── AppMessage ─────────────────────────────────────────────────────────────────

/// All messages the Fluid application can receive.
#[derive(Debug, Clone)]
pub enum AppMessage {
    // ── Pane grid ──────────────────────────────────────────────────────────
    PaneDragged(pane_grid::DragEvent),
    PaneResized(pane_grid::ResizeEvent),

    // ── Menu bar ───────────────────────────────────────────────────────────
    /// Toggle a top-level menu open/closed.
    MenuOpen(MenuTarget),
    /// Close all open menus (e.g. click outside).
    MenuClose,

    // ── Scene ──────────────────────────────────────────────────────────────
    Undo,
    Redo,

    // ── Simulation ─────────────────────────────────────────────────────────
    SimToggle,
    SimStep,
    SimReset,

    // ── Entity selection / mutation ────────────────────────────────────────
    SelectEntity(EntityId),
    RenameEntity(EntityId, String),
    MoveEntity(EntityId, [f32; 3]),

    // ── File ───────────────────────────────────────────────────────────────
    NewScene,
    /// Open a native file-open dialog and load the chosen `.fluid` file.
    OpenFileDialog,
    /// Open a native file-save dialog and save to the chosen path.
    SaveFileDialog,
    /// Save to a known path (reused when `current_path` is already set).
    SaveFile(PathBuf),
    /// Load scene from path (result of OpenFileDialog).
    OpenFile(PathBuf),
    /// Open a native dialog to import glTF/OBJ geometry.
    ImportFileDialog,
    /// Import geometry from the resolved path (result of ImportFileDialog).
    ImportFile(PathBuf),

    // ── Presets ────────────────────────────────────────────────────────────
    /// Apply a named material preset to the selected entity.
    LoadPreset(String),

    // ── Scene outliner ─────────────────────────────────────────────────────
    /// Spawn a new default entity via SpawnEntityCmd (DEC-015).
    SpawnEntity,

    // ── Debug server ───────────────────────────────────────────────────────
    DebugTick,

    #[allow(dead_code)]
    Noop,
}

// ── FluidApp ──────────────────────────────────────────────────────────────────

/// Top-level application state for the Fluid simulation GUI.
pub struct FluidApp {
    // ── Pane grid layout (DEC-010) ─────────────────────────────────────────
    /// The tiling pane layout — 5 panels (see `ui::layout`).
    panes: pane_grid::State<Panel>,

    // ── Shared debug-server state (DEC-009) ────────────────────────────────
    /// Frame-boundary snapshot written here; debug server reads via Arc.
    state_snapshot: Arc<RwLock<AppStateSnapshot>>,

    // ── Scene + undo history (DEC-015) ─────────────────────────────────────
    /// Undo/redo command history.  MUST exist before any scene mutation.
    command_history: CommandHistory,
    /// Active scene — all mutation via `command_history.execute(...)`.
    scene: Option<Scene>,

    // ── Viewport placeholder (session 4: wgpu) ────────────────────────────
    /// Viewport state stub — full wgpu integration is C8-Viewport (session 4).
    viewport: ViewportState,

    // ── Widget registry (DEC-013) ──────────────────────────────────────────
    /// Maps widget ID strings → metadata. Shared with the debug server.
    widget_registry: Arc<RwLock<WidgetRegistry>>,

    // ── Selection state (C8-UI) ────────────────────────────────────────────
    /// Currently selected entity, if any.
    selected_entity: Option<EntityId>,

    // ── Properties panel live-edit buffers ─────────────────────────────────
    /// Text buffer for the entity name input field.
    prop_name_buf: String,
    /// Text buffers for X/Y/Z position inputs.
    prop_pos_buf: [String; 3],

    // ── Frame counter ─────────────────────────────────────────────────────
    frame: u64,

    // ── Menu bar overlay state ─────────────────────────────────────────────
    /// Which top-level menu is currently open, if any.
    menu_open: Option<MenuTarget>,

    // ── File path tracking ─────────────────────────────────────────────────
    /// Most recently saved/opened path. Used by "Save" to skip the dialog.
    current_path: Option<PathBuf>,

    // ── Simulation state (DEC-006 Tier 0) ─────────────────────────────────
    sim_state: SimState,

    // ── Material preset database (C8-Assets) ──────────────────────────────
    preset_db: PresetDb,
}

impl FluidApp {
    /// Creates the initial application state.
    pub fn new(
        state_snapshot: Arc<RwLock<AppStateSnapshot>>,
        widget_registry: Arc<RwLock<WidgetRegistry>>,
    ) -> (Self, Task<AppMessage>) {
        let (panes, _viewport_pane) = build_pane_state();

        let app = Self {
            panes,
            state_snapshot,
            command_history: CommandHistory::default(),
            scene: None,
            viewport: ViewportState::default(),
            widget_registry,
            selected_entity: None,
            prop_name_buf: String::new(),
            prop_pos_buf: [String::new(), String::new(), String::new()],
            frame: 0,
            menu_open: None,
            current_path: None,
            sim_state: SimState::default(),
            preset_db: PresetDb::load(),
        };

        (app, Task::none())
    }

    // ── update ────────────────────────────────────────────────────────────

    /// Processes an `AppMessage` and returns any resulting `Task`.
    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            // ── Pane grid ───────────────────────────────────────────────────────────
            AppMessage::PaneDragged(event) => {
                if let pane_grid::DragEvent::Dropped { pane, target } = event {
                    self.panes.drop(pane, target);
                }
            }
            AppMessage::PaneResized(event) => {
                self.panes.resize(event.split, event.ratio);
            }

            // ── Menu bar ───────────────────────────────────────────────────────────
            AppMessage::MenuOpen(target) => {
                if self.menu_open == Some(target) {
                    self.menu_open = None; // toggle off
                } else {
                    self.menu_open = Some(target);
                }
            }
            AppMessage::MenuClose => {
                self.menu_open = None;
            }

            // ── Scene ─────────────────────────────────────────────────────
            AppMessage::Undo => {
                if let Some(scene) = self.scene.as_mut() {
                    if let Err(e) = self.command_history.undo(scene) {
                        log::warn!("Undo: {e}");
                    }
                }
            }
            AppMessage::Redo => {
                if let Some(scene) = self.scene.as_mut() {
                    if let Err(e) = self.command_history.redo(scene) {
                        log::warn!("Redo: {e}");
                    }
                }
            }

            // ── Simulation ─────────────────────────────────────────────────────────
            AppMessage::SimToggle => {
                self.sim_state.running = !self.sim_state.running;
                log::debug!("Sim toggled: running={}", self.sim_state.running);
            }
            AppMessage::SimStep => {
                self.sim_state.step();
                self.apply_sim_orbit();
            }
            AppMessage::SimReset => {
                self.sim_state.reset();
                // Return all entities to origin.
                if let Some(scene) = self.scene.as_mut() {
                    let entities: Vec<_> = scene.root_entities().to_vec();
                    for e in entities {
                        scene.set_position(e, [0.0, 0.0, 0.0]);
                    }
                }
                log::info!("Sim reset.");
            }

            // ── Entity selection / mutation ───────────────────────────────
            AppMessage::SelectEntity(id) => {
                self.selected_entity = Some(id);
                // Populate property buffers from current scene state.
                if let Some(scene) = self.scene.as_ref() {
                    if let Some(meta) = scene.meta(id) {
                        self.prop_name_buf = meta.name.clone();
                    }
                    let pos = scene.get_position(id);
                    self.prop_pos_buf = [
                        format!("{:.4}", pos[0]),
                        format!("{:.4}", pos[1]),
                        format!("{:.4}", pos[2]),
                    ];
                }
            }
            AppMessage::RenameEntity(id, name) => {
                // DEC-015: route through CommandHistory so rename is undoable.
                if let Some(scene) = self.scene.as_mut() {
                    let old_name = scene
                        .meta(id)
                        .map(|m| m.name.clone())
                        .unwrap_or_default();
                    let cmd = crate::scene::command::RenameEntityCmd::new(
                        id, old_name, name.clone(),
                    );
                    if let Err(e) = self.command_history.execute(cmd, scene) {
                        log::warn!("RenameEntity failed: {e}");
                    }
                }
                self.prop_name_buf = name;
            }
            AppMessage::MoveEntity(id, pos) => {
                // DEC-015: route through CommandHistory so move is undoable.
                if let Some(scene) = self.scene.as_mut() {
                    let old_pos = scene.get_position(id);
                    let cmd = crate::scene::command::MoveEntityCmd::new(id, old_pos, pos);
                    if let Err(e) = self.command_history.execute(cmd, scene) {
                        log::warn!("MoveEntity failed: {e}");
                    }
                }
                self.prop_pos_buf = [
                    format!("{:.4}", pos[0]),
                    format!("{:.4}", pos[1]),
                    format!("{:.4}", pos[2]),
                ];
            }

            // ── File ───────────────────────────────────────────────────────
            AppMessage::NewScene => {
                self.scene = Some(Scene::new());
                self.command_history.clear();
                self.selected_entity = None;
                self.current_path = None;
                self.sim_state.reset();
                log::info!("New scene created.");
            }
            AppMessage::OpenFileDialog => {
                self.menu_open = None;
                return Task::future(async {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("Fluid Scene", &["fluid"])
                        .pick_file()
                        .await;
                    match handle {
                        Some(h) => AppMessage::OpenFile(h.path().to_path_buf()),
                        None    => AppMessage::Noop,
                    }
                });
            }
            AppMessage::SaveFileDialog => {
                self.menu_open = None;
                if let Some(path) = self.current_path.clone() {
                    return Task::done(AppMessage::SaveFile(path));
                }
                return Task::future(async {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("Fluid Scene", &["fluid"])
                        .set_file_name("scene.fluid")
                        .save_file()
                        .await;
                    match handle {
                        Some(h) => AppMessage::SaveFile(h.path().to_path_buf()),
                        None    => AppMessage::Noop,
                    }
                });
            }
            AppMessage::SaveFile(path) => {
                if let Some(scene) = self.scene.as_ref() {
                    let mut envelope = FluidEnvelope::new(scene.name.clone());
                    envelope.entities = scene
                        .root_entities()
                        .iter()
                        .map(|&e| {
                            let name = scene
                                .meta(e)
                                .map(|m| m.name.clone())
                                .unwrap_or_else(|| format!("Entity {}", e.raw()));
                            let position = scene.get_position(e);
                            EntitySnapshot { id: e.raw(), name, position }
                        })
                        .collect();
                    match file::save(&path, &envelope) {
                        Ok(_) => {
                            log::info!("Saved to {:?}", path);
                            self.current_path = Some(path);
                        }
                        Err(e) => log::error!("Save failed: {e}"),
                    }
                }
            }
            AppMessage::OpenFile(path) => {
                match file::load(&path) {
                    Ok(envelope) => {
                        let mut scene = Scene::new();
                        scene.name = envelope.scene_name.clone();
                        for snap in &envelope.entities {
                            let e = scene.spawn_object(snap.name.clone());
                            scene.set_position(e, snap.position);
                        }
                        scene.mark_clean();
                        self.scene = Some(scene);
                        self.command_history.clear();
                        self.selected_entity = None;
                        self.current_path = Some(path.clone());
                        self.sim_state.reset();
                        log::info!("Opened {:?}", path);
                    }
                    Err(e) => log::error!("Open failed: {e}"),
                }
            }

            // ── Import ──────────────────────────────────────────────────────
            AppMessage::ImportFileDialog => {
                self.menu_open = None;
                return Task::future(async {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("glTF", &["gltf", "glb"])
                        .add_filter("All supported", &["gltf", "glb", "obj", "stl"])
                        .pick_file()
                        .await;
                    match handle {
                        Some(h) => AppMessage::ImportFile(h.path().to_path_buf()),
                        None    => AppMessage::Noop,
                    }
                });
            }
            AppMessage::ImportFile(path) => {
                if self.scene.is_none() {
                    self.scene = Some(Scene::new());
                }
                match crate::import::import_file(&path) {
                    Ok(meshes) => {
                        let count = meshes.len();
                        for mesh in meshes {
                            let cmd = crate::scene::command::SpawnEntityCmd::new(&mesh.name);
                            if let Some(scene) = self.scene.as_mut() {
                                let _ = self.command_history.execute(cmd, scene);
                                // Set position on the freshly-spawned entity.
                                if let Some(e) = scene.root_entities().last().copied() {
                                    scene.set_position(e, mesh.translation);
                                }
                            }
                        }
                        log::info!("Imported {count} mesh(es) from {:?}", path);
                    }
                    Err(e) => log::error!("Import failed: {e}"),
                }
            }

            // ── Presets ────────────────────────────────────────────────────
            AppMessage::LoadPreset(preset_name) => {
                if let Some(preset) = self.preset_db.get(&preset_name) {
                    log::info!(
                        "Applied preset \"{}\" (material={}, density={} kg/m³) to {:?}",
                        preset.name, preset.material, preset.density, self.selected_entity
                    );
                    // Full sim-parameter propagation is C8-SimBridge follow-up.
                    // For session 7 we log the application and mark the scene dirty.
                    if let Some(scene) = self.scene.as_mut() {
                        scene.mark_dirty();
                    }
                } else {
                    log::warn!("LoadPreset: unknown preset {:?}", preset_name);
                }
            }

            // ── Scene outliner ───────────────────────────────────────────────
            AppMessage::SpawnEntity => {
                if self.scene.is_none() {
                    self.scene = Some(Scene::new());
                }
                let name = format!(
                    "Object {}",
                    self.scene.as_ref().map(|s| s.root_entities().len()).unwrap_or(0) + 1
                );
                let cmd = crate::scene::command::SpawnEntityCmd::new(name);
                if let Some(scene) = self.scene.as_mut() {
                    if let Err(e) = self.command_history.execute(cmd, scene) {
                        log::warn!("SpawnEntity failed: {e}");
                    }
                }
            }

            // ── Debug tick ────────────────────────────────────────────────
            AppMessage::DebugTick => {
                self.frame += 1;
                // Advance sim tick and apply orbit if running.
                if self.sim_state.maybe_tick() {
                    self.apply_sim_orbit();
                }
                if let Ok(mut snap) = self.state_snapshot.write() {
                    snap.frame = self.frame;
                    snap.scene_name = self
                        .scene
                        .as_ref()
                        .map(|s| s.name.clone())
                        .unwrap_or_else(|| "—".to_string());
                    snap.entity_count = self
                        .scene
                        .as_ref()
                        .map(|s| s.root_entities().len() as u64)
                        .unwrap_or(0);
                }
            }

            AppMessage::Noop => {}
        }

        Task::none()
    }

    /// Applies placeholder orbit positions from `sim_state` to all root entities.
    /// This proves the sim → ECS → viewport → GPU pipeline without full physics.
    fn apply_sim_orbit(&mut self) {
        if let Some(scene) = self.scene.as_mut() {
            let entities: Vec<_> = scene.root_entities().to_vec();
            for (idx, entity) in entities.iter().enumerate() {
                let pos = self.sim_state.orbit_position(idx);
                scene.set_position(*entity, pos);
            }
        }
    }

    // ── view ──────────────────────────────────────────────────────────────

    /// Renders the full application UI.
    ///
    /// Layout: menu bar → pane grid → status bar.
    pub fn view(&self) -> Element<'_, AppMessage> {
        let menu   = self.view_menu_bar();
        let grid   = self.view_pane_grid();
        let status = self.view_status_bar();

        column![menu, grid, status]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    // ── theme ─────────────────────────────────────────────────────────────

    /// Returns the Fluid dark professional theme (DEC-001).
    pub fn theme(&self) -> Theme {
        fluid_theme()
    }

    // ── subscription ─────────────────────────────────────────────────────

    /// Returns the application subscription set.
    ///
    /// Current subscriptions:
    /// - Debug-tick every 100 ms (updates `AppStateSnapshot` for C9).
    ///
    /// DEC-017: Future file-watcher subscription MUST use
    /// `Subscription::run + stream::channel` — NOT bare threads.
    pub fn subscription(&self) -> Subscription<AppMessage> {
        // Debug-tick every 100 ms.
        let tick = iced::time::every(std::time::Duration::from_millis(100))
            .map(|_| AppMessage::DebugTick);

        // Keyboard shortcuts (DEC-001 iced 0.13 API).
        let keys = keyboard::on_key_press(|key, modifiers| {
            use keyboard::Key;
            if modifiers.command() {
                match key.as_ref() {
                    Key::Character("z") if modifiers.shift() => Some(AppMessage::Redo),
                    Key::Character("z") => Some(AppMessage::Undo),
                    Key::Character("y") => Some(AppMessage::Redo),
                    Key::Character("n") => Some(AppMessage::NewScene),
                    Key::Character("o") => Some(AppMessage::OpenFileDialog),
                    Key::Character("s") => Some(AppMessage::SaveFileDialog),
                    _ => None,
                }
            } else {
                None
            }
        });

        Subscription::batch([tick, keys])
    }

    // ── private view helpers ─────────────────────────────────────────────

    fn view_menu_bar(&self) -> Element<'_, AppMessage> {
        let bg        = Color::from_rgb8(0x0f, 0x0f, 0x13);
        let accent    = Color::from_rgb8(0x63, 0x66, 0xf1);
        let text_col  = Color::from_rgb8(0xe2, 0xe8, 0xf0);
        let muted     = Color::from_rgb8(0x88, 0x92, 0xa4);

        // The top bar with clickable menu labels.
        let bar = container(
            row![
                menu_button("File",       MenuTarget::File,       self.menu_open, accent, text_col),
                menu_button("Edit",       MenuTarget::Edit,       self.menu_open, accent, text_col),
                menu_button("Simulation", MenuTarget::Simulation, self.menu_open, accent, text_col),
                menu_button("View",       MenuTarget::View,       self.menu_open, accent, text_col),
                menu_button("Help",       MenuTarget::Help,       self.menu_open, accent, text_col),
                horizontal_space(),
                text(
                    self.scene
                        .as_ref()
                        .map(|s| format!("  {}{}  ", s.name, if s.dirty { " \u{25cf}" } else { "" }))
                        .unwrap_or_else(|| "  Fluid  ".to_string())
                )
                .size(12)
                .color(muted),
            ]
            .spacing(2)
            .padding([0, 8]),
        )
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(bg)),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fixed(28.0));

        // ── Shared dropdown style helpers ─────────────────────────────────
        let dropdown_bg   = Color::from_rgb8(0x1a, 0x1a, 0x24);
        let border_col    = Color::from_rgb8(0x2d, 0x2d, 0x3d);
        let hover_bg      = Color::from_rgba8(0x63, 0x66, 0xf1, 0.157_f32);
        let muted_col     = muted;

        // Helper: a normal enabled dropdown item.
        let item = |label: &'static str, msg: AppMessage| -> Element<'_, AppMessage> {
            button(text(label).size(12).color(text_col))
                .width(Length::Fill)
                .on_press(msg)
                .style(move |_t, s| button::Style {
                    background: Some(iced::Background::Color(
                        if matches!(s, button::Status::Hovered) { hover_bg }
                        else { Color::TRANSPARENT },
                    )),
                    text_color: text_col,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                })
                .into()
        };

        // Helper: a greyed-out (disabled) dropdown item.
        let item_disabled = |label: String| -> Element<'_, AppMessage> {
            button(text(label).size(12).color(muted_col))
                .width(Length::Fill)
                .style(move |_t, _s| button::Style {
                    background: Some(iced::Background::Color(Color::TRANSPARENT)),
                    text_color: muted_col,
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                })
                .into()
        };

        fn dd_style(dropdown_bg: Color, border_col: Color) -> container::Style {
            container::Style {
                background: Some(iced::Background::Color(dropdown_bg)),
                border: iced::Border {
                    color: border_col,
                    width: 1.0,
                    radius: iced::border::radius(4.0),
                },
                ..Default::default()
            }
        }

        if self.menu_open == Some(MenuTarget::File) {
            let dropdown = container(
                column![
                    item("New Scene\t Ctrl+N",    AppMessage::NewScene),
                    item("Open…\t Ctrl+O",        AppMessage::OpenFileDialog),
                    item("Import…",               AppMessage::ImportFileDialog),
                    item("Save\t Ctrl+S",         AppMessage::SaveFileDialog),
                    item("Save As…",              AppMessage::SaveFileDialog),
                ]
                .spacing(1)
                .padding([4, 0]),
            )
            .style(move |_t| dd_style(dropdown_bg, border_col))
            .width(Length::Fixed(200.0));
            column![
                bar,
                row![dropdown, horizontal_space()].padding([0, 8]),
            ]
            .spacing(0)
            .into()
        } else if self.menu_open == Some(MenuTarget::Edit) {
            // Build undo/redo labels dynamically.
            let undo_elem: Element<'_, AppMessage> =
                if self.command_history.can_undo() {
                    let label = format!(
                        "Undo {}\t Ctrl+Z",
                        self.command_history.peek_undo_label().unwrap_or("")
                    );
                    item(Box::leak(label.into_boxed_str()), AppMessage::Undo)
                } else {
                    item_disabled("Undo\t Ctrl+Z".to_string())
                };

            let redo_elem: Element<'_, AppMessage> =
                if self.command_history.can_redo() {
                    let label = format!(
                        "Redo {}\t Ctrl+Y",
                        self.command_history.peek_redo_label().unwrap_or("")
                    );
                    item(Box::leak(label.into_boxed_str()), AppMessage::Redo)
                } else {
                    item_disabled("Redo\t Ctrl+Y".to_string())
                };

            let dropdown = container(
                column![undo_elem, redo_elem]
                    .spacing(1)
                    .padding([4, 0]),
            )
            .style(move |_t| dd_style(dropdown_bg, border_col))
            .width(Length::Fixed(200.0));
            // Edit menu sits under the second button (~56 px from left).
            column![
                bar,
                row![
                    Space::with_width(56),
                    dropdown,
                    horizontal_space(),
                ]
                .padding([0, 8]),
            ]
            .spacing(0)
            .into()
        } else if self.menu_open == Some(MenuTarget::Simulation) {
            // Build preset items from the loaded database.
            let preset_names: Vec<&str> = self.preset_db.names();
            let preset_elems: Vec<Element<'_, AppMessage>> = if preset_names.is_empty() {
                vec![item_disabled("No presets loaded".to_string())]
            } else {
                preset_names
                    .iter()
                    .map(|&name| {
                        let owned = name.to_owned();
                        let label: &'static str = Box::leak(
                            format!("Apply \"{}\" preset", owned).into_boxed_str()
                        );
                        item(label, AppMessage::LoadPreset(owned))
                    })
                    .collect()
            };

            let dropdown = container(
                column(preset_elems).spacing(1).padding([4, 0]),
            )
            .style(move |_t| dd_style(dropdown_bg, border_col))
            .width(Length::Fixed(200.0));
            // Simulation sits under the third button (~115 px from left).
            column![
                bar,
                row![
                    Space::with_width(115),
                    dropdown,
                    horizontal_space(),
                ]
                .padding([0, 8]),
            ]
            .spacing(0)
            .into()
        } else {
            bar.into()
        }
    }

    fn view_pane_grid(&self) -> Element<'_, AppMessage> {
        pane_grid(&self.panes, |_pane_id, panel, _is_maximized| {
            let content = self.view_panel_content(panel);

            pane_grid::Content::new(
                container(content)
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(
                            Color::from_rgb8(0x1a, 0x1a, 0x24),
                        )),
                        border: iced::Border {
                            color: Color::from_rgb8(0x2d, 0x2d, 0x3d),
                            width: 1.0,
                            radius: iced::border::radius(0.0),
                        },
                        ..Default::default()
                    })
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .title_bar(
                pane_grid::TitleBar::new(
                    container(
                        text(panel.title()).size(12).color(Color::from_rgb8(0x88, 0x92, 0xa4)),
                    )
                    .padding([4, 8])
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(
                            Color::from_rgb8(0x0f, 0x0f, 0x13),
                        )),
                        ..Default::default()
                    }),
                )
                .padding(0),
            )
        })
        .on_drag(AppMessage::PaneDragged)
        .on_resize(6, AppMessage::PaneResized)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    /// Renders the content area for a given panel.
    fn view_panel_content<'a>(&'a self, panel: &Panel) -> Element<'a, AppMessage> {
        match panel {
            Panel::SceneOutliner => self.view_outliner_panel(),
            Panel::Viewport3D    => self.view_viewport_panel(),
            Panel::Properties    => self.view_properties_panel(),
            Panel::SimSetup      => self.view_sim_setup_panel(),
            Panel::Timeline      => self.view_timeline_panel(),
        }
    }

    fn view_outliner_panel(&self) -> Element<'_, AppMessage> {
        let accent      = Color::from_rgb8(0x63, 0x66, 0xf1);
        let muted       = Color::from_rgb8(0x88, 0x92, 0xa4);
        let text_col    = Color::from_rgb8(0xe2, 0xe8, 0xf0);
        // Selection highlight: accent at ~15% alpha over the panel background.
        let sel_bg      = Color::from_rgba8(0x63, 0x66, 0xf1, 0.15);
        let hover_bg    = Color::from_rgba8(0xff, 0xff, 0xff, 0.04);

        let entity_rows: Vec<Element<'_, AppMessage>> = self
            .scene
            .as_ref()
            .map(|s| {
                s.root_entities()
                    .iter()
                    .copied()
                    .map(|e| {
                        let meta_name = s
                            .meta(e)
                            .map(|m| m.name.as_str())
                            .unwrap_or("Entity")
                            .to_string();
                        let is_selected = self.selected_entity == Some(e);
                        let label = text(format!("  ▸ {meta_name}"))
                            .size(12)
                            .color(text_col);
                        let row_bg = if is_selected { sel_bg } else { Color::TRANSPARENT };
                        let btn = button(label)
                            .width(Length::Fill)
                            .on_press(AppMessage::SelectEntity(e))
                            .style(move |_theme, status| button::Style {
                                background: Some(iced::Background::Color(
                                    if matches!(status, button::Status::Hovered) && !is_selected {
                                        hover_bg
                                    } else {
                                        row_bg
                                    }
                                )),
                                text_color: text_col,
                                border: iced::Border::default(),
                                shadow: iced::Shadow::default(),
                            });
                        btn.into()
                    })
                    .collect()
            })
            .unwrap_or_default();

        let body: Element<'_, AppMessage> = if entity_rows.is_empty() {
            text("No objects in scene.")
                .size(11)
                .color(muted)
                .into()
        } else {
            scrollable(
                column(entity_rows).spacing(1).width(Length::Fill)
            )
            .height(Length::Fill)
            .into()
        };

        // "+" button in the outliner header — spawns a new default entity via DEC-015.
        let spawn_btn = button(text("+").size(12).color(accent))
            .on_press(AppMessage::SpawnEntity)
            .padding([0, 6])
            .style(move |_t, s| button::Style {
                background: Some(iced::Background::Color(
                    if matches!(s, button::Status::Hovered) {
                        Color::from_rgba8(0x63, 0x66, 0xf1, 0.15)
                    } else {
                        Color::TRANSPARENT
                    },
                )),
                text_color: accent,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            });

        container(
            column![
                row![
                    text("Objects").size(11).color(accent),
                    horizontal_space(),
                    spawn_btn,
                ]
                .align_y(iced::Alignment::Center),
                Space::with_height(4),
                body,
            ]
            .spacing(0)
            .padding([8, 6]),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_viewport_panel(&self) -> Element<'_, AppMessage> {
        // Construct a lightweight program snapshot for this frame.
        // Camera orbit/pan/zoom state lives inside the shader widget's
        // per-widget State (ViewportInteractState) — managed by iced.
        const MAX_POINTS: usize = 256; // matches pipeline::MAX_SPHERE_POINTS
        let positions: Vec<[f32; 3]> = self
            .scene
            .as_ref()
            .map(|s| {
                s.entity_positions(MAX_POINTS)
                    .into_iter()
                    .map(|(_, pos)| pos)
                    .collect()
            })
            .unwrap_or_default();
        let entity_count = positions.len() as u64;
        let program = ViewportProgram {
            entity_count,
            entity_positions: positions,
        };

        shader(program)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_properties_panel(&self) -> Element<'_, AppMessage> {
        let accent = Color::from_rgb8(0x63, 0x66, 0xf1);
        let muted  = Color::from_rgb8(0x88, 0x92, 0xa4);
        let _text_col = Color::from_rgb8(0xe2, 0xe8, 0xf0);

        let body: Element<'_, AppMessage> = match self.selected_entity {
            None => {
                text("Select an object to view its properties.")
                    .size(11)
                    .color(muted)
                    .into()
            }
            Some(entity_id) => {
                // Name input.
                let name_label = text("Name").size(10).color(muted);
                let name_input = text_input("Entity name", &self.prop_name_buf)
                    .size(12)
                    .on_input(move |s| AppMessage::RenameEntity(entity_id, s))
                    .on_submit(AppMessage::Noop);

                // Position inputs.
                let pos_label = text("Position").size(10).color(muted);
                let cur_pos: [f32; 3] = self
                    .scene
                    .as_ref()
                    .map(|s| s.get_position(entity_id))
                    .unwrap_or([0.0, 0.0, 0.0]);

                let axis_labels = ["X", "Y", "Z"];
                let pos_row: Element<'_, AppMessage> = row(
                    axis_labels
                        .iter()
                        .enumerate()
                        .map(|(i, &ax)| {
                            let buf = &self.prop_pos_buf[i];
                            let axis_label = text(ax)
                                .size(10)
                                .color(muted)
                                .width(Length::Fixed(12.0));
                            let input = text_input("", buf)
                                .size(11)
                                .width(Length::Fill)
                                .on_input(move |s| {
                                    // Build new position array without mutation.
                                    if let Ok(v) = s.parse::<f32>() {
                                        let new_pos = [
                                            if i == 0 { v } else { cur_pos[0] },
                                            if i == 1 { v } else { cur_pos[1] },
                                            if i == 2 { v } else { cur_pos[2] },
                                        ];
                                        AppMessage::MoveEntity(entity_id, new_pos)
                                    } else {
                                        AppMessage::Noop
                                    }
                                });
                            row![axis_label, input]
                                .spacing(4)
                                .width(Length::Fill)
                                .into()
                        })
                        .collect::<Vec<_>>()
                )
                .spacing(6)
                .into();


                column![
                    name_label,
                    Space::with_height(2),
                    name_input,
                    Space::with_height(8),
                    pos_label,
                    Space::with_height(2),
                    pos_row,
                ]
                .spacing(0)
                .into()
            }
        };

        container(
            column![
                text("Properties").size(11).color(accent),
                Space::with_height(8),
                body,
            ]
            .spacing(0)
            .padding([8, 8]),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_sim_setup_panel(&self) -> Element<'_, AppMessage> {
        container(
            column![
                text("Simulation").size(11).color(Color::from_rgb8(0x63, 0x66, 0xf1)),
                Space::with_height(8),
                text("No simulation configured.")
                    .size(11)
                    .color(Color::from_rgb8(0x88, 0x92, 0xa4)),
            ]
            .spacing(0)
            .padding([8, 8]),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_timeline_panel(&self) -> Element<'_, AppMessage> {
        let accent   = Color::from_rgb8(0x63, 0x66, 0xf1);
        let text_col = Color::from_rgb8(0xe2, 0xe8, 0xf0);
        let muted    = Color::from_rgb8(0x88, 0x92, 0xa4);

        let play_label = if self.sim_state.running { "⏸" } else { "▶" };
        // Short labels — "Tick 9999" wraps vertically in a 20%-height panel.
        let time_str = format!("t={:.2}s", self.sim_state.time());
        let tick_str = format!("#{}", self.sim_state.tick);

        let mk_btn_style = move |col: Color| {
            move |_theme: &Theme, _s: button::Status| button::Style {
                background: None,
                text_color: col,
                border: iced::Border::default(),
                shadow: iced::Shadow::default(),
            }
        };

        container(
            row![
                button(text(play_label).size(14).color(accent))
                    .on_press(AppMessage::SimToggle)
                    .padding([2, 6])
                    .style(mk_btn_style(accent)),
                button(text("⏭").size(12).color(muted))
                    .on_press(AppMessage::SimStep)
                    .padding([2, 4])
                    .style(mk_btn_style(muted)),
                button(text("⏹").size(12).color(muted))
                    .on_press(AppMessage::SimReset)
                    .padding([2, 4])
                    .style(mk_btn_style(muted)),
                Space::with_width(8),
                text(time_str).size(11).color(text_col),
                Space::with_width(8),
                text(tick_str).size(11).color(muted),
                horizontal_space(),
            ]
            .spacing(2)
            .align_y(iced::Alignment::Center)
            .padding([0, 8]),
        )
        .center_y(Length::Fill)
        .width(Length::Fill)
        .into()
    }

    fn view_status_bar(&self) -> Element<'_, AppMessage> {
        let bg     = Color::from_rgb8(0x0f, 0x0f, 0x13);
        let muted  = Color::from_rgb8(0x88, 0x92, 0xa4);
        let border = Color::from_rgb8(0x2d, 0x2d, 0x3d);

        // Build an owned String so it can be moved into text() without a borrow of self.
        let tier_str: String = self
            .state_snapshot
            .read()
            .ok()
            .map(|s| format!("Tier {}", s.tier))
            .unwrap_or_else(|| "Tier ?".to_string());

        let frame_str = format!("Frame {}", self.frame);

        container(
            row![
                text("Fluid").size(11).color(Color::from_rgb8(0x63, 0x66, 0xf1)),
                Space::with_width(16),
                text(tier_str).size(11).color(muted),
                horizontal_space(),
                text(frame_str).size(11).color(muted),
            ]
            .spacing(0)
            .padding([0, 10]),
        )
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(bg)),
            border: iced::Border {
                color: border,
                width: 1.0,
                radius: iced::border::radius(0.0),
            },
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fixed(22.0))
        .center_y(Length::Fixed(22.0))
        .into()
    }
}

// ── Widget helpers ────────────────────────────────────────────────────────────

/// Renders a clickable menu bar button that dispatches `MenuOpen(target)`.
/// Highlighted (accent underline) when that menu is currently open.
fn menu_button<'a>(
    label: &'a str,
    target: MenuTarget,
    open: Option<MenuTarget>,
    accent: Color,
    text_col: Color,
) -> Element<'a, AppMessage> {
    let is_open = open == Some(target);
    let col = if is_open { accent } else { text_col };
    button(
        text(label).size(12).color(col)
    )
    .on_press(AppMessage::MenuOpen(target))
    .padding([4, 8])
    .style(move |_theme, _status| button::Style {
        background: if is_open {
            Some(iced::Background::Color(Color::from_rgba8(0x63, 0x66, 0xf1, 0.094_f32)))
        } else {
            None
        },
        text_color: col,
        border: iced::Border::default(),
        shadow: iced::Shadow::default(),
    })
    .into()
}
