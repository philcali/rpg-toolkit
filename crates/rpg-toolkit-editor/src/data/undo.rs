//! Undo/redo history management.
//!
//! This module provides the `UndoHistory` resource which maintains two bounded stacks
//! (undo and redo) of `EditCommand`s, enabling reversible editing operations.

use bevy::prelude::*;
use rpg_toolkit_common::MapData;

use super::commands::EditCommand;

/// Undo/redo history resource. Maintains two stacks capped at `max_history`.
#[derive(Resource)]
pub struct UndoHistory {
    pub undo_stack: Vec<EditCommand>,
    pub redo_stack: Vec<EditCommand>,
    pub max_history: usize,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_history: 50,
        }
    }
}

impl UndoHistory {
    /// Pushes a command onto the undo stack, clears the redo stack,
    /// and enforces the maximum history size.
    pub fn push_command(&mut self, cmd: EditCommand) {
        self.redo_stack.clear();
        self.undo_stack.push(cmd);
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Undoes the most recent command. Returns `true` if an undo was performed.
    pub fn undo(&mut self, map: &mut MapData) -> bool {
        if let Some(cmd) = self.undo_stack.pop() {
            cmd.apply_inverse(map);
            self.redo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    /// Redoes the most recently undone command. Returns `true` if a redo was performed.
    pub fn redo(&mut self, map: &mut MapData) -> bool {
        if let Some(cmd) = self.redo_stack.pop() {
            cmd.apply(map);
            self.undo_stack.push(cmd);
            true
        } else {
            false
        }
    }
}
