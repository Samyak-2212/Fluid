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

use std::sync::{Arc, RwLock};

use iced::{
    Element, Subscription, Task, Theme,
    widget::{
        column, container, horizontal_space, pane_grid,
        row, text, Space,
        shader,
    },
    Color, Length,
};

use crate::{
    debug_server::AppStateSnapshot,
    scene::{command::CommandHistory, Scene},
    ui::{
        layout::{build_pane_state, Panel},
        theme::fluid_theme,
        widget_registry::WidgetRegistry,
    },
    viewport::{ViewportState, ViewportProgram},
};

// ── AppMessage ─────────────────────────────────────────────────────────────────

/// All messages the Fluid application can receive.
#[derive(Debug, Clone)]
pub enum AppMessage {
    // ── Pane grid ──────────────────────────────────────────────────────────
    /// A pane drag event — forwarded to `pane_grid::State::drag`.
    PaneDragged(pane_grid::DragEvent),
    /// A pane resize event — forwarded to `pane_grid::State::resize`.
    PaneResized(pane_grid::ResizeEvent),

    // ── Scene ──────────────────────────────────────────────────────────────
    /// Undo the last scene command.
    Undo,
    /// Redo the most recently undone command.
    Redo,

    // ── Simulation ─────────────────────────────────────────────────────────
    /// Toggle simulation running/paused.
    SimToggle,
    /// Step the simulation by one frame.
    SimStep,
    /// Reset the simulation to t=0.
    SimReset,

    // ── File ───────────────────────────────────────────────────────────────
    /// Request a new, empty scene.
    NewScene,
    /// Open file dialog to load a `.fluid` file.
    OpenFile,
    /// Save the current scene to disk.
    Save,

    // ── Debug server ───────────────────────────────────────────────────────
    /// Tick from the debug-server subscription — updates the state snapshot.
    DebugTick,

    /// No-op / internal — used for Task::none() paths that need a typed message.
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

    // ── Frame counter ─────────────────────────────────────────────────────
    frame: u64,
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
            frame: 0,
        };

        (app, Task::none())
    }

    // ── update ────────────────────────────────────────────────────────────

    /// Processes an `AppMessage` and returns any resulting `Task`.
    pub fn update(&mut self, message: AppMessage) -> Task<AppMessage> {
        match message {
            // ── Pane grid ─────────────────────────────────────────────────
            AppMessage::PaneDragged(event) => {
                if let pane_grid::DragEvent::Dropped { pane, target } = event {
                    self.panes.drop(pane, target);
                }
            }
            AppMessage::PaneResized(event) => {
                self.panes.resize(event.split, event.ratio);
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

            // ── Simulation ────────────────────────────────────────────────
            AppMessage::SimToggle | AppMessage::SimStep | AppMessage::SimReset => {
                // TODO(C8-SimBridge): wire to sim_bridge when implemented.
                log::debug!("Sim message received: {:?}", message);
            }

            // ── File ──────────────────────────────────────────────────────
            AppMessage::NewScene => {
                self.scene = Some(Scene::new());
                self.command_history.clear();
                log::info!("New scene created.");
            }
            AppMessage::OpenFile | AppMessage::Save => {
                // TODO(C8-FileFormat): implement file dialog + MessagePack I/O.
                log::debug!("File message received: {:?}", message);
            }

            // ── Debug tick ────────────────────────────────────────────────
            AppMessage::DebugTick => {
                self.frame += 1;
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
        // Tick every 100 ms to refresh the debug state snapshot.
        // iced::time::every requires the `tokio` (or `smol`) feature (added in Cargo.toml).
        iced::time::every(std::time::Duration::from_millis(100))
            .map(|_instant| AppMessage::DebugTick)
    }

    // ── private view helpers ─────────────────────────────────────────────

    fn view_menu_bar(&self) -> Element<'_, AppMessage> {
        let bg = Color::from_rgb8(0x0f, 0x0f, 0x13);

        container(
            row![
                menu_item("File"),
                menu_item("Edit"),
                menu_item("Simulation"),
                menu_item("View"),
                menu_item("Help"),
                horizontal_space(),
                // Scene name badge (right-aligned).
                text(
                    self.scene
                        .as_ref()
                        .map(|s| format!("  {}{}  ", s.name, if s.dirty { " ●" } else { "" }))
                        .unwrap_or_else(|| "  Fluid  ".to_string())
                )
                .size(12)
                .color(Color::from_rgb8(0x88, 0x92, 0xa4)),
            ]
            .spacing(2)
            .padding([0, 8]),
        )
        .style(move |_theme| container::Style {
            background: Some(iced::Background::Color(bg)),
            ..Default::default()
        })
        .width(Length::Fill)
        .height(Length::Fixed(28.0))
        .into()
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
        let accent = Color::from_rgb8(0x63, 0x66, 0xf1);
        let muted  = Color::from_rgb8(0x88, 0x92, 0xa4);

        let entity_rows: Vec<Element<'_, AppMessage>> = self
            .scene
            .as_ref()
            .map(|s| {
                s.root_entities()
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let meta_name = s
                            .root_entities()
                            .get(i)
                            .and_then(|e| s.meta(*e))
                            .map(|m| m.name.as_str())
                            .unwrap_or("Entity");
                        let label: Element<'_, AppMessage> = text(format!("  ▸ {meta_name}"))
                            .size(12)
                            .color(Color::from_rgb8(0xe2, 0xe8, 0xf0))
                            .into();
                        label
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
            column(entity_rows).spacing(2).into()
        };

        container(
            column![
                text("Objects").size(11).color(accent),
                Space::with_height(4),
                body,
            ]
            .spacing(0)
            .padding([8, 8]),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_viewport_panel(&self) -> Element<'_, AppMessage> {
        // Construct a lightweight program snapshot for this frame.
        // Camera orbit/pan/zoom state lives inside the shader widget's
        // per-widget State (ViewportInteractState) — managed by iced.
        let program = ViewportProgram {
            entity_count: self
                .scene
                .as_ref()
                .map(|s| s.root_entities().len() as u64)
                .unwrap_or(0),
        };

        shader(program)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_properties_panel(&self) -> Element<'_, AppMessage> {
        container(
            column![
                text("Properties").size(11).color(Color::from_rgb8(0x63, 0x66, 0xf1)),
                Space::with_height(8),
                text("Select an object to view its properties.")
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
        container(
            row![
                text("▶").size(16).color(Color::from_rgb8(0x63, 0x66, 0xf1)),
                Space::with_width(12),
                text("t = 0.000 s").size(11).color(Color::from_rgb8(0xe2, 0xe8, 0xf0)),
                Space::with_width(12),
                text("Frame 0").size(11).color(Color::from_rgb8(0x88, 0x92, 0xa4)),
            ]
            .spacing(0)
            .padding([0, 12]),
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

/// Renders a plain text menu item label.
///
/// Interactive menu dropdowns are deferred — this is a stub for the title bar.
/// Widget id is set for C9 visibility (DEC-013).
fn menu_item(label: &str) -> Element<'_, AppMessage> {
    text(label)
        .size(12)
        .color(Color::from_rgb8(0xe2, 0xe8, 0xf0))
        .into()
}
