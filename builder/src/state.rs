// builder/src/state.rs
//
// Build state machine. Tracks per-component build status and elapsed time.

use std::time::{Duration, Instant};

/// Per-component build status.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ComponentStatus {
    /// Not scheduled for build.
    Pending,
    /// Currently building.
    Building { started_at: Instant },
    /// Completed successfully.
    Succeeded { elapsed: Duration },
    /// Failed with exit code.
    Failed { exit_code: Option<i32>, elapsed: Duration },
    /// Cancelled by user.
    Cancelled,
}

#[allow(dead_code)]
impl ComponentStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            ComponentStatus::Succeeded { .. }
                | ComponentStatus::Failed { .. }
                | ComponentStatus::Cancelled
        )
    }

    /// Returns elapsed time if available.
    pub fn elapsed(&self) -> Option<Duration> {
        match self {
            ComponentStatus::Building { started_at } => Some(started_at.elapsed()),
            ComponentStatus::Succeeded { elapsed } => Some(*elapsed),
            ComponentStatus::Failed { elapsed, .. } => Some(*elapsed),
            _ => None,
        }
    }
}

/// Overall build session state.
#[derive(Debug, Clone, PartialEq)]
pub enum BuildSessionState {
    Idle,
    Running,
    Finished,
    Cancelled,
}

/// Holds state for the entire build session including live output lines.
pub struct BuildState {
    pub session: BuildSessionState,
    pub output_lines: Vec<String>,
    pub component_statuses: std::collections::HashMap<String, ComponentStatus>,
}

impl BuildState {
    pub fn new() -> Self {
        BuildState {
            session: BuildSessionState::Idle,
            output_lines: Vec::new(),
            component_statuses: std::collections::HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.session = BuildSessionState::Idle;
        self.output_lines.clear();
        self.component_statuses.clear();
    }

    pub fn push_output(&mut self, line: String) {
        self.output_lines.push(line);
        // Cap at 10 000 lines to avoid unbounded memory growth.
        if self.output_lines.len() > 10_000 {
            self.output_lines.drain(0..1_000);
        }
    }
}

impl Default for BuildState {
    fn default() -> Self {
        Self::new()
    }
}
