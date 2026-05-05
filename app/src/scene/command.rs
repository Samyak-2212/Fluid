//! Undo/redo command pattern for all scene mutations.
//!
//! **This module MUST be implemented before any code that mutates scene state.**
//! (DEC-015 — locked architectural decision)
//!
//! # Design
//! All mutations to the scene graph go through [`SceneCommand`]. Commands are
//! stored in [`CommandHistory`] which maintains a linear undo stack and a redo
//! list. This ensures every user action is reversible and serialisable for
//! future macro-recording and scripting.
//!
//! # Usage
//! ```rust,ignore
//! // Within the Iced update loop:
//! let cmd = AddObjectCommand::new(object_id, transform);
//! history.execute(cmd, &mut scene)?;
//!
//! // Undo the last action:
//! history.undo(&mut scene)?;
//!
//! // Redo the previously undone action:
//! history.redo(&mut scene)?;
//! ```

use std::fmt;

// ── Error type ───────────────────────────────────────────────────────────────

/// Errors that can occur during command execution or undo/redo.
#[derive(Debug)]
pub enum SceneCommandError {
    /// The command failed to apply. Contains a human-readable reason.
    ExecuteError(String),
    /// The command failed to reverse. State may be inconsistent — caller should
    /// consider this a logic error and log/report it.
    UndoError(String),
    /// Nothing to undo (history is empty or at the beginning).
    NothingToUndo,
    /// Nothing to redo (at the tip of the history stack).
    NothingToRedo,
}

impl fmt::Display for SceneCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExecuteError(msg) => write!(f, "command execute failed: {msg}"),
            Self::UndoError(msg) => write!(f, "command undo failed: {msg}"),
            Self::NothingToUndo => write!(f, "nothing to undo"),
            Self::NothingToRedo => write!(f, "nothing to redo"),
        }
    }
}

impl std::error::Error for SceneCommandError {}

// ── SceneCommand trait ───────────────────────────────────────────────────────

/// A reversible scene mutation.
///
/// Implementors represent a single atomic change to the scene (e.g. add object,
/// set transform, change material). Every command must implement both `execute`
/// and `undo` to maintain history integrity.
///
/// # Thread safety
/// Commands are `Send + Sync` because they may be queued from background threads
/// (e.g. file-load tasks) and consumed on the main/UI thread.
pub trait SceneCommand: Send + Sync + fmt::Debug {
    /// Returns a short human-readable label shown in the Undo/Redo menu (e.g.
    /// "Add Object", "Set Transform"). MUST NOT contain newlines.
    fn label(&self) -> &str;

    /// Applies the command to `scene`. Must be idempotent when called exactly
    /// once after construction. Do not call `execute` more than once on the
    /// same command instance.
    fn execute(&mut self, scene: &mut crate::scene::Scene) -> Result<(), SceneCommandError>;

    /// Reverses the effect of the most recent `execute` call. Called at most
    /// once per `execute`. After `undo`, calling `execute` again must restore
    /// the same effect (redo path).
    fn undo(&mut self, scene: &mut crate::scene::Scene) -> Result<(), SceneCommandError>;
}

// ── CommandHistory ───────────────────────────────────────────────────────────

/// Bounded linear undo/redo history for scene commands.
///
/// The history behaves like a cursor in a list of commands:
/// - Executing a new command appends it to the undo stack and clears the redo list.
/// - Undo moves the cursor back one step (reverses the command).
/// - Redo moves the cursor forward one step (re-executes the command).
///
/// The history is bounded by [`CommandHistory::max_depth`] entries. When the
/// limit is exceeded, the oldest entries are dropped.
pub struct CommandHistory {
    /// Commands that can be undone. Index 0 = oldest; last = most recent.
    undo_stack: Vec<Box<dyn SceneCommand>>,
    /// Commands that can be redone. Index 0 = next to redo.
    redo_stack: Vec<Box<dyn SceneCommand>>,
    /// Maximum number of undoable entries kept in memory.
    max_depth: usize,
}

impl fmt::Debug for CommandHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandHistory")
            .field("undo_depth", &self.undo_stack.len())
            .field("redo_depth", &self.redo_stack.len())
            .field("max_depth", &self.max_depth)
            .finish()
    }
}

impl Default for CommandHistory {
    fn default() -> Self {
        Self::new(200)
    }
}

impl CommandHistory {
    /// Creates a new empty history with the given undo depth limit.
    ///
    /// A `max_depth` of `200` is the application default. Set lower for
    /// memory-constrained scenarios (Tier 0 devices).
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_depth.min(64)),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    /// Executes `cmd` against `scene`, then stores it on the undo stack.
    ///
    /// Any pending redo entries are discarded (branching history is not
    /// supported — redo is linear).
    ///
    /// If the undo stack would exceed [`Self::max_depth`], the oldest entry
    /// is dropped before appending.
    pub fn execute(
        &mut self,
        mut cmd: Box<dyn SceneCommand>,
        scene: &mut crate::scene::Scene,
    ) -> Result<(), SceneCommandError> {
        cmd.execute(scene)?;
        // New command branches the history — drop stale redo entries.
        self.redo_stack.clear();
        // Enforce depth limit.
        if self.undo_stack.len() >= self.max_depth {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(cmd);
        Ok(())
    }

    /// Undoes the most recently executed command.
    ///
    /// Returns [`SceneCommandError::NothingToUndo`] if the history is empty.
    pub fn undo(
        &mut self,
        scene: &mut crate::scene::Scene,
    ) -> Result<(), SceneCommandError> {
        let mut cmd = self.undo_stack.pop().ok_or(SceneCommandError::NothingToUndo)?;
        cmd.undo(scene)?;
        self.redo_stack.push(cmd);
        Ok(())
    }

    /// Redoes the most recently undone command.
    ///
    /// Returns [`SceneCommandError::NothingToRedo`] if there is nothing to redo.
    pub fn redo(
        &mut self,
        scene: &mut crate::scene::Scene,
    ) -> Result<(), SceneCommandError> {
        let mut cmd = self.redo_stack.pop().ok_or(SceneCommandError::NothingToRedo)?;
        cmd.execute(scene)?;
        self.undo_stack.push(cmd);
        Ok(())
    }

    /// Returns the label of the command that would be undone next, if any.
    pub fn peek_undo_label(&self) -> Option<&str> {
        self.undo_stack.last().map(|c| c.label())
    }

    /// Returns the label of the command that would be redone next, if any.
    pub fn peek_redo_label(&self) -> Option<&str> {
        self.redo_stack.last().map(|c| c.label())
    }

    /// Returns `true` if there is at least one command available to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if there is at least one command available to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clears all undo and redo history. Use when loading a new scene.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Returns a snapshot of recent undo labels for the UI Undo menu.
    /// Labels are ordered newest-first (index 0 = next to undo).
    pub fn undo_labels(&self, limit: usize) -> Vec<&str> {
        self.undo_stack
            .iter()
            .rev()
            .take(limit)
            .map(|c| c.label())
            .collect()
    }
}

// ── Concrete command impls (DEC-015) ─────────────────────────────────────────

use fluid_core::ecs::traits::EntityId;

/// Renames a scene entity. Stores old and new names for undo/redo.
#[derive(Debug)]
pub struct RenameEntityCmd {
    pub entity: EntityId,
    pub old_name: String,
    pub new_name: String,
}

impl RenameEntityCmd {
    pub fn new(entity: EntityId, old_name: String, new_name: String) -> Box<Self> {
        Box::new(Self { entity, old_name, new_name })
    }
}

impl SceneCommand for RenameEntityCmd {
    fn label(&self) -> &str { "Rename Entity" }

    fn execute(&mut self, scene: &mut crate::scene::Scene) -> Result<(), SceneCommandError> {
        if let Some(meta) = scene.meta_mut(self.entity) {
            meta.name = self.new_name.clone();
            scene.mark_dirty();
            Ok(())
        } else {
            Err(SceneCommandError::ExecuteError(format!(
                "RenameEntityCmd: entity {:?} not found", self.entity
            )))
        }
    }

    fn undo(&mut self, scene: &mut crate::scene::Scene) -> Result<(), SceneCommandError> {
        if let Some(meta) = scene.meta_mut(self.entity) {
            meta.name = self.old_name.clone();
            scene.mark_dirty();
            Ok(())
        } else {
            Err(SceneCommandError::UndoError(format!(
                "RenameEntityCmd: entity {:?} not found on undo", self.entity
            )))
        }
    }
}

/// Moves a scene entity to a new world-space position. Stores old and new positions for undo/redo.
#[derive(Debug)]
pub struct MoveEntityCmd {
    pub entity: EntityId,
    pub old_pos: [f32; 3],
    pub new_pos: [f32; 3],
}

impl MoveEntityCmd {
    pub fn new(entity: EntityId, old_pos: [f32; 3], new_pos: [f32; 3]) -> Box<Self> {
        Box::new(Self { entity, old_pos, new_pos })
    }
}

impl SceneCommand for MoveEntityCmd {
    fn label(&self) -> &str { "Move Entity" }

    fn execute(&mut self, scene: &mut crate::scene::Scene) -> Result<(), SceneCommandError> {
        scene.set_position(self.entity, self.new_pos);
        Ok(())
    }

    fn undo(&mut self, scene: &mut crate::scene::Scene) -> Result<(), SceneCommandError> {
        scene.set_position(self.entity, self.old_pos);
        Ok(())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::Scene;

    // Minimal no-op command for testing history mechanics.
    #[derive(Debug)]
    struct NoOp {
        label: &'static str,
        executed: u32,
        undone: u32,
    }

    impl NoOp {
        fn new(label: &'static str) -> Box<Self> {
            Box::new(Self { label, executed: 0, undone: 0 })
        }
    }

    impl SceneCommand for NoOp {
        fn label(&self) -> &str { self.label }

        fn execute(&mut self, _scene: &mut Scene) -> Result<(), SceneCommandError> {
            self.executed += 1;
            Ok(())
        }

        fn undo(&mut self, _scene: &mut Scene) -> Result<(), SceneCommandError> {
            self.undone += 1;
            Ok(())
        }
    }

    fn empty_scene() -> Scene {
        Scene::new()
    }

    #[test]
    fn history_empty_undo_returns_nothing_to_undo() {
        let mut h = CommandHistory::new(10);
        let mut s = empty_scene();
        let err = h.undo(&mut s).unwrap_err();
        assert!(matches!(err, SceneCommandError::NothingToUndo));
    }

    #[test]
    fn history_empty_redo_returns_nothing_to_redo() {
        let mut h = CommandHistory::new(10);
        let mut s = empty_scene();
        let err = h.redo(&mut s).unwrap_err();
        assert!(matches!(err, SceneCommandError::NothingToRedo));
    }

    #[test]
    fn execute_then_undo_then_redo() {
        let mut h = CommandHistory::new(10);
        let mut s = empty_scene();

        h.execute(NoOp::new("A"), &mut s).unwrap();
        assert!(h.can_undo());
        assert!(!h.can_redo());
        assert_eq!(h.peek_undo_label(), Some("A"));

        h.undo(&mut s).unwrap();
        assert!(!h.can_undo());
        assert!(h.can_redo());

        h.redo(&mut s).unwrap();
        assert!(h.can_undo());
        assert!(!h.can_redo());
    }

    #[test]
    fn new_command_clears_redo_stack() {
        let mut h = CommandHistory::new(10);
        let mut s = empty_scene();

        h.execute(NoOp::new("A"), &mut s).unwrap();
        h.undo(&mut s).unwrap();
        assert!(h.can_redo());

        // Executing B must drop the redo entry for A.
        h.execute(NoOp::new("B"), &mut s).unwrap();
        assert!(!h.can_redo());
        assert_eq!(h.peek_undo_label(), Some("B"));
    }

    #[test]
    fn depth_limit_drops_oldest() {
        let mut h = CommandHistory::new(3);
        let mut s = empty_scene();

        for name in ["A", "B", "C", "D"] {
            h.execute(NoOp::new(name), &mut s).unwrap();
        }
        // Only the 3 most recent are kept.
        let labels = h.undo_labels(10);
        assert_eq!(labels, vec!["D", "C", "B"]);
    }

    #[test]
    fn clear_resets_both_stacks() {
        let mut h = CommandHistory::new(10);
        let mut s = empty_scene();

        h.execute(NoOp::new("A"), &mut s).unwrap();
        h.undo(&mut s).unwrap();
        h.clear();
        assert!(!h.can_undo());
        assert!(!h.can_redo());
    }
}
