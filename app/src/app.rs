//! `FluidApp` — the top-level Iced application struct.
//!
//! Implements the four methods required by `iced::application`:
//!   - `update(&mut self, msg: AppMessage) -> iced::Task<AppMessage>`
//!   - `view(&self) -> iced::Element<'_, AppMessage>`
//!   - `theme(&self) -> iced::Theme`
//!   - `subscription(&self) -> iced::Subscription<AppMessage>`
//!
//! # Menu overlay (BUG-1 fix)
//! Dropdown menus are rendered as a `stack!` overlay above the main layout.
//! This prevents the old inline-column approach from stealing height from the
//! viewport pane. The menu bar strip is always exactly MENU_BAR_H pixels.
//!
//! # Widget IDs (DEC-013)
//! Every interactive widget MUST call `.id(iced::widget::Id::new("…"))`.
//! Missing IDs are invisible to C9.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use iced::{
    Element, Subscription, Task, Theme,
    keyboard,
    widget::{
        button, column, container, horizontal_space, mouse_area, pane_grid,
        row, scrollable, stack, text, text_input, Space,
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

// ── Layout constants ─────────────────────────────────────────────────────────

/// Height of the menu bar strip in pixels.
const MENU_BAR_H: f32 = 28.0;

/// Approximate pixel offsets for each menu button's left edge.
/// These are estimates based on text width + padding at 12px font.
/// True pixel-perfect alignment requires iced's advanced layout API.
const MENU_OFFSET_FILE:       f32 =   8.0;   // "File" starts near left edge
const MENU_OFFSET_EDIT:       f32 =  56.0;   // after "File" button (~48px wide)
const MENU_OFFSET_SIMULATION: f32 = 108.0;   // after "Edit" button (~52px wide)
const MENU_OFFSET_VIEW:       f32 = 204.0;   // after "Simulation" button (~96px)
const MENU_OFFSET_HELP:       f32 = 252.0;   // after "View" button (~48px)

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
    /// Close all open menus (e.g. click outside or item selected).
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
    /// Open a native dialog to import glTF/OBJ/STL geometry.
    ImportFileDialog,
    /// Import geometry from the resolved path (result of ImportFileDialog).
    ImportFile(PathBuf),

    // ── Presets ────────────────────────────────────────────────────────────
    /// Apply a named material preset to the selected entity.
    LoadPreset(String),

    // ── Scene outliner ─────────────────────────────────────────────────────
    /// Spawn a new default entity via SpawnEntityCmd (DEC-015).
    SpawnEntity,
    /// Delete the currently selected entity via DespawnEntityCmd (DEC-015).
    DeleteEntity,

    // ── Status ─────────────────────────────────────────────────────────────
    /// Clears the transient status message in the status bar.
    ClearStatus,

    // ── Debug server ───────────────────────────────────────────────────────
    DebugTick,

    // ── Autosave ───────────────────────────────────────────────────────────
    /// Fired by the autosave timer; saves if dirty + path known.
    AutosaveTick,

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
    /// Undo/redo command history. MUST exist before any scene mutation.
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

    // ── Transient status message ───────────────────────────────────────────
    /// Shown in the status bar; cleared after a short delay via ClearStatus.
    status_message: Option<String>,
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
            status_message: None,
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
                self.menu_open = None;
                log::debug!("Sim toggled: running={}", self.sim_state.running);
            }
            AppMessage::SimStep => {
                self.sim_state.step();
                self.apply_sim_orbit();
                self.menu_open = None;
            }
            AppMessage::SimReset => {
                self.sim_state.reset();
                self.menu_open = None;
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
                self.menu_open = None;
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
                            self.set_status("Scene saved.");
                        }
                        Err(e) => {
                            log::error!("Save failed: {e}");
                            self.set_status(format!("Save failed: {e}"));
                        }
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
                        let count = scene.root_entities().len();
                        self.scene = Some(scene);
                        self.command_history.clear();
                        self.selected_entity = None;
                        self.current_path = Some(path.clone());
                        self.sim_state.reset();
                        self.set_status(format!("Opened — {count} object(s)"));
                        log::info!("Opened {:?}", path);
                    }
                    Err(e) => {
                        log::error!("Open failed: {e}");
                        self.set_status(format!("Open failed: {e}"));
                    }
                }
            }

            // ── Import ──────────────────────────────────────────────────────
            AppMessage::ImportFileDialog => {
                self.menu_open = None;
                return Task::future(async {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("glTF", &["gltf", "glb"])
                        .add_filter("OBJ", &["obj"])
                        .add_filter("STL", &["stl"])
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
                        if count == 0 {
                            self.set_status("Import: file contained no mesh data.");
                            log::warn!("ImportFile: {:?} contained no meshes", path);
                        } else {
                            let mut first_entity: Option<EntityId> = None;
                            for (idx, mesh) in meshes.into_iter().enumerate() {
                                // Spread meshes that all land at origin along X
                                // so they are individually visible in the viewport.
                                let translation = if mesh.translation == [0.0, 0.0, 0.0]
                                    && idx > 0
                                {
                                    [idx as f32 * 1.5, 0.0, 0.0]
                                } else {
                                    mesh.translation
                                };

                                if let Some(scene) = self.scene.as_mut() {
                                    // Direct spawn for import (undo of bulk-import
                                    // deferred to future work — DEC-015 relaxation
                                    // for batch operations).
                                    let entity = scene.spawn_object(mesh.name.clone());
                                    scene.set_position(entity, translation);
                                    if first_entity.is_none() {
                                        first_entity = Some(entity);
                                    }
                                }
                            }
                            // Select first imported entity and set status.
                            let fname = path
                                .file_name()
                                .map(|s| s.to_string_lossy().into_owned())
                                .unwrap_or_else(|| format!("{:?}", path));
                            self.set_status(format!("Imported {count} mesh(es) from {fname}"));
                            log::info!("Imported {count} mesh(es) from {:?}", path);
                            if let Some(e) = first_entity {
                                return Task::done(AppMessage::SelectEntity(e));
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Import failed: {e}");
                        self.set_status(format!("Import failed: {e}"));
                    }
                }
            }

            // ── Status ─────────────────────────────────────────────────────
            AppMessage::ClearStatus => {
                self.status_message = None;
            }

            // ── Presets ────────────────────────────────────────────────────
            AppMessage::LoadPreset(preset_name) => {
                self.menu_open = None;
                if let Some(preset) = self.preset_db.get(&preset_name) {
                    log::info!(
                        "Applied preset \"{}\" (material={}, density={} kg/m³) to {:?}",
                        preset.name, preset.material, preset.density, self.selected_entity
                    );
                    // C8-SimBridge: insert SimParameters into ECS world for selected entity.
                    if let (Some(entity_id), Some(scene)) =
                        (self.selected_entity, self.scene.as_mut())
                    {
                        use std::any::TypeId;
                        let params = crate::sim_bridge::SimParameters {
                            viscosity: preset.viscosity,
                            density:   preset.density,
                            material:  preset.material.clone(),
                        };
                        scene.world_mut().insert_erased(
                            entity_id,
                            TypeId::of::<crate::sim_bridge::SimParameters>(),
                            Box::new(params),
                        );
                        scene.mark_dirty();
                        self.set_status(format!("Preset \"{}\" applied.", preset.name));
                    } else {
                        log::warn!("LoadPreset: no entity selected");
                        self.set_status("Select an object first.");
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
            AppMessage::DeleteEntity => {
                if let (Some(id), Some(scene)) = (self.selected_entity, self.scene.as_mut()) {
                    let name = scene.meta(id).map(|m| m.name.clone()).unwrap_or_default();
                    let pos  = scene.get_position(id);
                    let cmd  = crate::scene::command::DespawnEntityCmd::new(id, name, pos);
                    if let Err(e) = self.command_history.execute(cmd, scene) {
                        log::warn!("DeleteEntity failed: {e}");
                    }
                    self.selected_entity = None;
                    self.prop_name_buf.clear();
                    self.prop_pos_buf = [String::new(), String::new(), String::new()];
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

            // ── Autosave tick ─────────────────────────────────────────────
            AppMessage::AutosaveTick => {
                // Only save if the scene is dirty and a path is already known.
                if let (Some(path), Some(scene)) = (self.current_path.clone(), self.scene.as_ref()) {
                    if scene.dirty {
                        log::debug!("Autosave firing for {:?}", path);
                        return Task::done(AppMessage::SaveFile(path));
                    }
                }
            }

            AppMessage::Noop => {}
        }

        Task::none()
    }

    /// Sets a transient status message and schedules its clearance after 4 seconds.
    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
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
    /// Layout: menu bar strip (fixed 28px) → pane grid (fills) → status bar.
    ///
    /// When a menu is open the dropdown is rendered as a `stack!` overlay so
    /// it does NOT steal height from the pane grid (BUG-1 fix).
    pub fn view(&self) -> Element<'_, AppMessage> {
        let menu_strip = self.view_menu_bar_strip();
        let grid       = self.view_pane_grid();
        let status     = self.view_status_bar();

        let base: Element<'_, AppMessage> = column![menu_strip, grid, status]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if self.menu_open.is_none() {
            return base;
        }

        // ── Build overlay layers ──────────────────────────────────────────
        // Layer 1 (dismiss): transparent mouse_area covering everything below
        // the menu bar. Pressing it sends MenuClose.
        let dismiss: Element<'_, AppMessage> = column![
            Space::with_height(MENU_BAR_H),
            mouse_area(Space::new(Length::Fill, Length::Fill))
                .on_press(AppMessage::MenuClose),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        // Layer 2 (dropdown): positioned via column + row spacing.
        let (dropdown_elem, x_offset) = self.view_open_dropdown();
        let dropdown_layer: Element<'_, AppMessage> = column![
            Space::with_height(MENU_BAR_H),
            row![
                Space::with_width(x_offset),
                dropdown_elem,
                horizontal_space(),
            ],
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

        // stack!: base → dismiss → dropdown (topmost receives events first).
        stack![base, dismiss, dropdown_layer].into()
    }

    // ── theme ─────────────────────────────────────────────────────────────

    /// Returns the Fluid dark professional theme (DEC-001).
    pub fn theme(&self) -> Theme {
        fluid_theme()
    }

    // ── subscription ─────────────────────────────────────────────────────

    /// Returns the application subscription set.
    ///
    /// DEC-017: file-watcher subscription MUST use
    /// `Subscription::run + stream::channel` — NOT bare threads.
    pub fn subscription(&self) -> Subscription<AppMessage> {
        // Debug-tick every 100 ms.
        let tick = iced::time::every(std::time::Duration::from_millis(100))
            .map(|_| AppMessage::DebugTick);

        // Autosave every 60 s (DEC-017: Subscription, not bare thread).
        let autosave = iced::time::every(std::time::Duration::from_secs(60))
            .map(|_| AppMessage::AutosaveTick);

        // Keyboard shortcuts (DEC-001 iced 0.13 API).
        let keys = keyboard::on_key_press(|key, modifiers| {
            use keyboard::Key;
            use keyboard::key::Named;
            if modifiers.command() {
                match key.as_ref() {
                    Key::Character("z") if modifiers.shift() => Some(AppMessage::Redo),
                    Key::Character("z") => Some(AppMessage::Undo),
                    Key::Character("y") => Some(AppMessage::Redo),
                    Key::Character("n") => Some(AppMessage::NewScene),
                    Key::Character("o") => Some(AppMessage::OpenFileDialog),
                    Key::Character("s") => Some(AppMessage::SaveFileDialog),
                    Key::Character("i") => Some(AppMessage::ImportFileDialog),
                    _ => None,
                }
            } else {
                // Non-modifier keys.
                match key.as_ref() {
                    Key::Named(Named::Delete)    => Some(AppMessage::DeleteEntity),
                    Key::Named(Named::Escape)    => Some(AppMessage::MenuClose),
                    Key::Character(" ")          => Some(AppMessage::SimToggle),
                    _ => None,
                }
            }
        });

        Subscription::batch([tick, autosave, keys])
    }

    // ── private view helpers ─────────────────────────────────────────────

    /// Renders just the 28px menu bar strip (buttons only, no dropdown).
    /// The dropdown is rendered separately as a stack overlay in `view()`.
    fn view_menu_bar_strip(&self) -> Element<'_, AppMessage> {
        let bg       = Color::from_rgb8(0x0f, 0x0f, 0x13);
        let accent   = Color::from_rgb8(0x63, 0x66, 0xf1);
        let text_col = Color::from_rgb8(0xe2, 0xe8, 0xf0);
        let muted    = Color::from_rgb8(0x88, 0x92, 0xa4);

        container(
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
        .height(Length::Fixed(MENU_BAR_H))
        .into()
    }

    /// Builds the dropdown content for the currently open menu.
    /// Returns `(element, x_offset_pixels)` where the x_offset positions the
    /// dropdown below the correct menu button.
    fn view_open_dropdown(&self) -> (Element<'_, AppMessage>, f32) {
        let text_col    = Color::from_rgb8(0xe2, 0xe8, 0xf0);
        let muted       = Color::from_rgb8(0x88, 0x92, 0xa4);
        let accent      = Color::from_rgb8(0x63, 0x66, 0xf1);
        let dropdown_bg = Color::from_rgb8(0x1a, 0x1a, 0x24);
        let border_col  = Color::from_rgb8(0x2d, 0x2d, 0x3d);
        let hover_bg    = Color::from_rgba8(0x63, 0x66, 0xf1, 0.157_f32);

        let target = match self.menu_open {
            Some(t) => t,
            None    => return (Space::new(0, 0).into(), 0.0),
        };

        let x_offset = match target {
            MenuTarget::File       => MENU_OFFSET_FILE,
            MenuTarget::Edit       => MENU_OFFSET_EDIT,
            MenuTarget::Simulation => MENU_OFFSET_SIMULATION,
            MenuTarget::View       => MENU_OFFSET_VIEW,
            MenuTarget::Help       => MENU_OFFSET_HELP,
        };

        let items: Vec<Element<'_, AppMessage>> = match target {
            // ── File menu ─────────────────────────────────────────────────
            MenuTarget::File => vec![
                dd_item_shortcut("New Scene",  "Ctrl+N", AppMessage::NewScene,         text_col, muted, hover_bg),
                dd_item_shortcut("Open...",    "Ctrl+O", AppMessage::OpenFileDialog,   text_col, muted, hover_bg),
                dd_item_shortcut("Import...",  "Ctrl+I", AppMessage::ImportFileDialog, text_col, muted, hover_bg),
                dd_separator(border_col),
                dd_item_shortcut("Save",       "Ctrl+S", AppMessage::SaveFileDialog,   text_col, muted, hover_bg),
                dd_item("Save As...",                    AppMessage::SaveFileDialog,   text_col, hover_bg),
            ],

            // ── Edit menu ─────────────────────────────────────────────────
            MenuTarget::Edit => {
                let undo_elem = if self.command_history.can_undo() {
                    let label = format!(
                        "Undo {}",
                        self.command_history.peek_undo_label().unwrap_or("")
                    );
                    dd_item_shortcut(label, "Ctrl+Z", AppMessage::Undo, text_col, muted, hover_bg)
                } else {
                    dd_item_disabled_shortcut("Undo", "Ctrl+Z", muted)
                };
                let redo_elem = if self.command_history.can_redo() {
                    let label = format!(
                        "Redo {}",
                        self.command_history.peek_redo_label().unwrap_or("")
                    );
                    dd_item_shortcut(label, "Ctrl+Y", AppMessage::Redo, text_col, muted, hover_bg)
                } else {
                    dd_item_disabled_shortcut("Redo", "Ctrl+Y", muted)
                };
                vec![
                    undo_elem,
                    redo_elem,
                    dd_separator(border_col),
                    dd_item("Spawn Object",  AppMessage::SpawnEntity,  text_col, hover_bg),
                    if self.selected_entity.is_some() {
                        dd_item_shortcut("Delete Object", "Del", AppMessage::DeleteEntity, text_col, muted, hover_bg)
                    } else {
                        dd_item_disabled_shortcut("Delete Object", "Del", muted)
                    },
                ]
            }

            // ── Simulation menu ───────────────────────────────────────────
            MenuTarget::Simulation => {
                let play_label: &str = if self.sim_state.running {
                    "|| Pause"
                } else {
                    "> Run"
                };
                let mut items: Vec<Element<'_, AppMessage>> = vec![
                    dd_item_shortcut(play_label, "Space", AppMessage::SimToggle, accent,   muted, hover_bg),
                    dd_item(">> Step",                                           AppMessage::SimStep,    text_col, hover_bg),
                    dd_item("[] Reset",                                          AppMessage::SimReset,   text_col, hover_bg),
                    dd_separator(border_col),
                    dd_label("Material presets:", muted),
                ];
                let preset_names = self.preset_db.names();
                if preset_names.is_empty() {
                    items.push(dd_item_disabled("No presets loaded", muted));
                } else {
                    for name in preset_names {
                        let label  = format!("Apply \"{}\"", name);
                        let msg    = AppMessage::LoadPreset(name.to_owned());
                        items.push(dd_item(label, msg, text_col, hover_bg));
                    }
                }
                items
            }

            // ── View menu ─────────────────────────────────────────────────
            MenuTarget::View => vec![
                dd_item_disabled("Grid (always on)", muted),
                dd_item_disabled("Axes (always on)", muted),
                dd_separator(border_col),
                dd_item_disabled("Detach Panel (v2)", muted),
                dd_item_disabled("Reset Layout (v2)", muted),
            ],

            // ── Help menu ─────────────────────────────────────────────────
            MenuTarget::Help => vec![
                dd_item_disabled("Documentation (v2)", muted),
                dd_item_disabled("Report Issue (v2)",  muted),
                dd_separator(border_col),
                dd_item_disabled("About Fluid", muted),
            ],
        };

        let dropdown = container(
            column(items).spacing(1).padding([4, 0]),
        )
        .style(move |_t| container::Style {
            background: Some(iced::Background::Color(dropdown_bg)),
            border: iced::Border {
                color: border_col,
                width: 1.0,
                radius: iced::border::radius(4.0),
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.35),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            ..Default::default()
        })
        .width(Length::Fixed(220.0));

        (dropdown.into(), x_offset)
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
        let sel_bg      = Color::from_rgba8(0x63, 0x66, 0xf1, 0.149_f32);
        let hover_bg    = Color::from_rgba8(0xff, 0xff, 0xff, 0.039_f32);

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
                        let label = text(format!("  \u{25b8} {meta_name}"))
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
                        Color::from_rgba8(0x63, 0x66, 0xf1, 0.149_f32)
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
        let accent = Color::from_rgb8(0x63, 0x66, 0xf1);
        let muted  = Color::from_rgb8(0x88, 0x92, 0xa4);

        let preset_rows: Vec<Element<'_, AppMessage>> = {
            let names = self.preset_db.names();
            if names.is_empty() {
                vec![text("No presets loaded.").size(11).color(muted).into()]
            } else {
                names.iter().map(|&name| {
                    let msg = AppMessage::LoadPreset(name.to_owned());
                    let label_str = name.to_owned();
                    let is_sel = self.selected_entity.is_some();
                    button(text(label_str).size(11).color(if is_sel { Color::from_rgb8(0xe2, 0xe8, 0xf0) } else { muted }))
                        .width(Length::Fill)
                        .on_press_maybe(if is_sel { Some(msg) } else { None })
                        .style(move |_t, s| button::Style {
                            background: Some(iced::Background::Color(
                                if matches!(s, button::Status::Hovered) && is_sel {
                                    Color::from_rgba8(0x63, 0x66, 0xf1, 0.149_f32)
                                } else {
                                    Color::TRANSPARENT
                                }
                            )),
                            text_color: if is_sel { Color::from_rgb8(0xe2, 0xe8, 0xf0) } else { muted },
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                        })
                        .into()
                }).collect()
            }
        };

        let hint = if self.selected_entity.is_none() {
            text("Select an object to apply a preset.").size(10).color(muted)
        } else {
            text("Click preset to apply to selected object.").size(10).color(muted)
        };

        container(
            column![
                text("Simulation").size(11).color(accent),
                Space::with_height(4),
                hint,
                Space::with_height(8),
                text("Material Presets").size(10).color(muted),
                Space::with_height(4),
                scrollable(column(preset_rows).spacing(2)).height(Length::Fill),
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

        let play_label = if self.sim_state.running { "\u{23f8}" } else { "\u{25b6}" };
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
                button(text("\u{23ed}").size(12).color(muted))
                    .on_press(AppMessage::SimStep)
                    .padding([2, 4])
                    .style(mk_btn_style(muted)),
                button(text("\u{23f9}").size(12).color(muted))
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
        let accent = Color::from_rgb8(0x63, 0x66, 0xf1);
        let muted  = Color::from_rgb8(0x88, 0x92, 0xa4);
        let border = Color::from_rgb8(0x2d, 0x2d, 0x3d);
        let ok_col = Color::from_rgb8(0x4a, 0xde, 0x80); // green for success messages

        // Tier string — read from snapshot to avoid holding the lock across view.
        let tier_str: String = self
            .state_snapshot
            .read()
            .ok()
            .map(|s| format!("Tier {}", s.tier))
            .unwrap_or_else(|| "Tier ?".to_string());

        let frame_str = format!("Frame {}", self.frame);

        // Sim state indicator.
        let (sim_label, sim_col) = if self.sim_state.running {
            ("\u{25b6} Running", accent)
        } else {
            ("\u{23f8} Paused", muted)
        };

        // Entity count.
        let entity_count = self
            .scene
            .as_ref()
            .map(|s| s.root_entities().len())
            .unwrap_or(0);
        let entity_str = format!(
            "{} object{}",
            entity_count,
            if entity_count == 1 { "" } else { "s" }
        );

        // Transient status message (shown on right side, green text).
        let status_elem: Element<'_, AppMessage> = if let Some(msg) = &self.status_message {
            text(msg.as_str()).size(11).color(ok_col).into()
        } else {
            Space::with_width(0).into()
        };

        container(
            row![
                text("Fluid").size(11).color(accent),
                Space::with_width(16),
                text(tier_str).size(11).color(muted),
                Space::with_width(16),
                text(sim_label).size(11).color(sim_col),
                Space::with_width(12),
                text(entity_str).size(11).color(muted),
                horizontal_space(),
                status_elem,
                Space::with_width(16),
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

// ── Standalone widget helpers ─────────────────────────────────────────────────

/// Renders a clickable menu bar button that dispatches `MenuOpen(target)`.
/// Highlighted (accent color) when that menu is currently open.
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

/// A dropdown menu item — full-width button with hover highlight.
/// Accepts any `Into<String>` to avoid `Box::leak` (BUG-3 fix).
fn dd_item<'a>(
    label: impl Into<String>,
    msg: AppMessage,
    text_col: Color,
    hover_bg: Color,
) -> Element<'a, AppMessage> {
    button(text(label.into()).size(12).color(text_col))
        .width(Length::Fill)
        .on_press(msg)
        .padding([4, 12])
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
}

/// A greyed-out (non-interactive) dropdown label.
fn dd_item_disabled<'a>(label: impl Into<String>, muted: Color) -> Element<'a, AppMessage> {
    container(text(label.into()).size(12).color(muted))
        .width(Length::Fill)
        .padding([4, 12])
        .into()
}

/// A small non-interactive section label (e.g. "Material presets:").
fn dd_label<'a>(label: impl Into<String>, muted: Color) -> Element<'a, AppMessage> {
    container(text(label.into()).size(10).color(muted))
        .width(Length::Fill)
        .padding([2, 12])
        .into()
}

/// A 1px horizontal separator line for dropdown menus.
fn dd_separator<'a>(border_col: Color) -> Element<'a, AppMessage> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .style(move |_t| container::Style {
            background: Some(iced::Background::Color(border_col)),
            ..Default::default()
        })
        .padding([0, 0])
        .height(Length::Fixed(1.0))
        .into()
}

/// Dropdown item with label on the left and keyboard shortcut right-aligned
/// in muted colour. Avoids tab characters which iced renders as U+FFFD.
fn dd_item_shortcut<'a>(
    label:    impl Into<String>,
    shortcut: &'static str,
    msg:      AppMessage,
    text_col: Color,
    muted:    Color,
    hover_bg: Color,
) -> Element<'a, AppMessage> {
    let inner = row![
        text(label.into()).size(12).color(text_col),
        horizontal_space(),
        text(shortcut).size(11).color(muted),
    ]
    .align_y(iced::Alignment::Center);

    button(inner)
        .width(Length::Fill)
        .on_press(msg)
        .padding([4, 12])
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
}

/// Disabled dropdown item with label and shortcut, both in muted colour.
fn dd_item_disabled_shortcut<'a>(
    label:    impl Into<String>,
    shortcut: &'static str,
    muted:    Color,
) -> Element<'a, AppMessage> {
    let inner = row![
        text(label.into()).size(12).color(muted),
        horizontal_space(),
        text(shortcut).size(11).color(muted),
    ]
    .align_y(iced::Alignment::Center);

    container(inner)
        .width(Length::Fill)
        .padding([4, 12])
        .into()
}
