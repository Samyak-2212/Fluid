//! Pane-grid tiling layout for the Fluid application.
//!
//! Implements the 5-panel layout described in `coordinators/app/PROMPT.md`:
//! - Scene Outliner (left)
//! - 3D Viewport (center, large)
//! - Properties Panel (right)
//! - Sim Setup Panel (lower-left)
//! - Timeline / Playback Bar (lower-center)
//!
//! Panel resize and drag-to-reorder are provided by `iced::pane_grid` (DEC-010).
//! Floating panel detach is deferred to v2.
//!
//! # Layout tree
//! The initial pane split is constructed in [`build_pane_state`].
//! The returned `pane_grid::State<Panel>` is owned by `FluidApp`.

use iced::widget::pane_grid;

/// Panel identity — used as the pane_grid state key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Panel {
    SceneOutliner,
    Viewport3D,
    Properties,
    SimSetup,
    Timeline,
}

impl Panel {
    /// Returns the human-readable panel title for the header bar.
    pub fn title(&self) -> &'static str {
        match self {
            Panel::SceneOutliner => "Scene",
            Panel::Viewport3D    => "Viewport",
            Panel::Properties    => "Properties",
            Panel::SimSetup      => "Simulation",
            Panel::Timeline      => "Timeline",
        }
    }
}

/// Builds the initial `pane_grid::State<Panel>` for the 5-panel Fluid layout.
///
/// Split strategy (all ratios approximate):
/// ```text
/// ┌──────────┬───────────────────────────┬───────────────┐
/// │ Outliner │      Viewport3D           │  Properties   │
/// │  (left)  │    (center, ~55 %)        │   (right)     │
/// ├──────────┤                           │               │
/// │ SimSetup │                           │               │
/// │(lower-L) ├───────────────────────────┤               │
/// │          │       Timeline            │               │
/// │          │     (lower-center)        │               │
/// └──────────┴───────────────────────────┴───────────────┘
/// ```
///
/// Returns the `State` and a handle to the Viewport pane.
pub fn build_pane_state() -> (pane_grid::State<Panel>, pane_grid::Pane) {
    // ── Layout strategy ───────────────────────────────────────────────────────
    // iced pane_grid::State::split(axis, pane, content):
    //   • The ORIGINAL pane stays as the FIRST child (left / top).
    //   • The NEW pane becomes the SECOND child (right / bottom).
    // resize(split, ratio):
    //   • `ratio` is the fraction given to the FIRST (original) pane.
    //
    // Desired layout:
    //   [SceneOutliner(20%) | Viewport(55%) | Properties(25%)]
    //   Viewport splits horizontally: top=Viewport(80%), bottom=Timeline(20%)
    //   Outliner splits horizontally: top=Outliner(60%), bottom=SimSetup(40%)
    // ─────────────────────────────────────────────────────────────────────────

    // Start with SceneOutliner as the root pane so it sits on the far LEFT.
    let (mut state, outliner_pane) = pane_grid::State::new(Panel::SceneOutliner);

    // ── Step 1: add Viewport to the RIGHT of Outliner.
    // split(outliner) → outliner stays FIRST(left), Viewport is SECOND(right).
    // resize ratio 0.20 → Outliner=20%, Viewport=80%.
    let (viewport_pane, vp_split) = state
        .split(pane_grid::Axis::Vertical, outliner_pane, Panel::Viewport3D)
        .expect("split Outliner|Viewport");
    state.resize(vp_split, 0.20);

    // ── Step 2: add Properties to the RIGHT of Viewport.
    // split(viewport) → Viewport stays FIRST(left), Properties is SECOND(right).
    // We have 80% total for [Viewport + Properties].
    // Want Viewport≈55% and Properties≈25% of total.
    // ratio = 55/80 ≈ 0.6875 → Viewport=55%, Properties=25%.
    let (_properties_pane, props_split) = state
        .split(pane_grid::Axis::Vertical, viewport_pane, Panel::Properties)
        .expect("split Viewport|Properties");
    state.resize(props_split, 0.6875);

    // ── Step 3: split Outliner horizontally → Outliner(top 60%) / SimSetup(bot 40%).
    // split(outliner) → Outliner stays FIRST(top), SimSetup is SECOND(bottom).
    // ratio 0.60 → Outliner=60%, SimSetup=40%.
    let (_sim_pane, sim_split) = state
        .split(pane_grid::Axis::Horizontal, outliner_pane, Panel::SimSetup)
        .expect("split Outliner|SimSetup");
    state.resize(sim_split, 0.60);

    // ── Step 4: split Viewport horizontally → Viewport(top 80%) / Timeline(bot 20%).
    // split(viewport) → Viewport stays FIRST(top), Timeline is SECOND(bottom).
    // ratio 0.80 → Viewport=80%, Timeline=20%.
    let (_timeline_pane, timeline_split) = state
        .split(pane_grid::Axis::Horizontal, viewport_pane, Panel::Timeline)
        .expect("split Viewport|Timeline");
    state.resize(timeline_split, 0.80);

    (state, viewport_pane)
}
