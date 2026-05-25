//! Simulation bridge — in-process Tier 0 (Rayon) + subprocess Tier 1–3 (DEC-006).
//!
//! This module holds the `SimState` control struct and a minimal in-process
//! Tier 0 tick that animates entity positions in a placeholder orbit to prove
//! the sim → scene → viewport → GPU pipeline is live.
//!
//! Full physics integration (C8-SimBridge sub-coordinator) is a later session.

pub mod tier_select;

/// Simulation execution tier selected at runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimTier {
    /// In-process Rayon thread pool. Always available. Preview quality.
    Tier0,
    /// Subprocess. Requires pre-compiled `fluid_sim` Tier 1 binary in `app/bin/`.
    Tier1,
    /// Subprocess. Requires Tier 2 binary.
    Tier2,
    /// Subprocess (HPC Pack). Optional install.
    Tier3,
}

/// In-process simulation state for Tier 0 preview.
///
/// Controls the running flag, tick counter, and time step.
/// Actual physics is a placeholder orbit — the goal is to prove the
/// sim → ECS → viewport → GPU pipeline without full physics.
#[derive(Debug, Clone)]
pub struct SimState {
    /// Whether the simulation is currently advancing.
    pub running: bool,
    /// Current simulation tick count.
    pub tick: u64,
    /// Fixed time step per tick, in seconds.
    pub dt: f32,
}

impl Default for SimState {
    fn default() -> Self {
        Self { running: false, tick: 0, dt: 0.016_7 } // ~60 Hz
    }
}

impl SimState {
    /// Returns the elapsed simulation time in seconds.
    pub fn time(&self) -> f32 {
        self.tick as f32 * self.dt
    }

    /// Advances the simulation by one tick if `running`.
    ///
    /// Returns `true` if a tick was performed (caller should update scene).
    pub fn maybe_tick(&mut self) -> bool {
        if self.running {
            self.tick += 1;
            true
        } else {
            false
        }
    }

    /// Forces exactly one tick regardless of `running` state.
    pub fn step(&mut self) {
        self.tick += 1;
    }

    /// Resets the simulation to `tick = 0`, stops running.
    pub fn reset(&mut self) {
        self.tick = 0;
        self.running = false;
    }

    /// Computes a placeholder orbit position for a given entity index.
    ///
    /// Rotates around the Y axis at a rate proportional to `tick`, offset by
    /// the entity index so multiple entities form a visible spread.
    /// Radius = `2.0 + entity_index as f32 * 1.5` metres.
    ///
    /// This is a placeholder only — not physics. Replace in the full sim bridge.
    pub fn orbit_position(&self, entity_index: usize) -> [f32; 3] {
        let angle = self.tick as f32 * 0.01 + entity_index as f32 * std::f32::consts::TAU
            / 6.0_f32.max(entity_index as f32 + 1.0);
        let radius = 2.0 + entity_index as f32 * 1.5;
        [
            radius * angle.cos(),
            (self.tick as f32 * 0.005 + entity_index as f32 * 0.3).sin() * 0.5,
            radius * angle.sin(),
        ]
    }
}

/// Sim bridge state placeholder.
#[derive(Debug, Default)]
pub struct SimBridge {
    pub active_tier: Option<SimTier>,
}

// ── SimParameters component ───────────────────────────────────────────────────

/// ECS component that stores the active simulation material parameters for an
/// entity, set when the user applies a material preset via `LoadPreset`.
///
/// Stored in the `WorldAny` ECS store via `insert_erased` so that downstream
/// physics integrators can read them without a direct dependency on `app/`.
///
/// C8-SimBridge: this is the first real data path from UI → ECS.
/// Full physics propagation (reading these values in physics_core) is a future session.
#[derive(Debug, Clone)]
pub struct SimParameters {
    /// Dynamic viscosity in Pa·s (0.0 for solids).
    pub viscosity: f64,
    /// Material density in kg/m³.
    pub density: f64,
    /// Human-readable material class: "liquid", "gas", or "solid".
    pub material: String,
}
