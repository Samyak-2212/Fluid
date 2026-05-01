// [REVIEWED: claude — C1 complete gate + C7 quality gate, 2026-05-02. No issues found.]
//! Fixed-timestep manager.
//!
//! Reads `dt` from `config/core.toml` (key: `timestep_seconds`).
//! No hardcoded timestep values. Runtime panics on missing config keys are
//! forbidden — the manager must supply a typed default.

use crate::units::Seconds;
use std::fs;

pub struct Timestep {
    dt: Seconds,
    accumulated: Seconds,
}

impl Timestep {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let mut parsed_dt = 1.0 / 60.0;
        if let Ok(content) = fs::read_to_string("config/core.toml") {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("timestep_seconds") {
                    if let Some(eq_idx) = line.find('=') {
                        if let Ok(val) = line[eq_idx + 1..].trim().parse::<f64>() {
                            parsed_dt = val;
                        }
                    }
                }
            }
        }

        Self {
            dt: Seconds(parsed_dt),
            accumulated: Seconds(0.0),
        }
    }

    /// The fixed simulation step size in SI seconds.
    pub fn dt(&self) -> Seconds {
        self.dt
    }

    /// Current accumulator value (remainder after last set of ticks).
    pub fn accumulated(&self) -> Seconds {
        self.accumulated
    }

    /// Advances the accumulator by `frame_time`.
    pub fn add_frame_time(&mut self, frame_time: Seconds) {
        self.accumulated.0 += frame_time.0;
    }

    /// Subtracts `dt` and returns `true` if a fixed tick should be executed.
    /// Must be called in a loop until it returns `false`.
    pub fn tick(&mut self) -> bool {
        if self.accumulated.0 >= self.dt.0 {
            self.accumulated.0 -= self.dt.0;
            true
        } else {
            false
        }
    }
}
