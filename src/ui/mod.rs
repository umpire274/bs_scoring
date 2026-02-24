pub mod cli;
pub mod events;
pub mod tui;

use crate::ui::events::UiEvent;

/// UI abstraction layer.
///
/// - CLI implementation prints directly.
/// - Future TUI implementation will keep a scrollable log buffer and render a fixed prompt line.
pub trait Ui {
    fn emit(&mut self, event: UiEvent);

    /// Read a single command line from the user.
    ///
    /// `prompt` is a fully formatted prompt string (dynamic, derived from game state).
    fn read_command_line(&mut self, prompt: &str) -> Option<String>;
}
