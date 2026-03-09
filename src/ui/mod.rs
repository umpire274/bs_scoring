pub mod app;
pub mod cli;
pub mod context;
pub mod events;
pub mod factory;
pub mod tui;

use crate::models::play_ball::GameState;
use crate::ui::events::UiEvent;
pub use app::App;
pub use context::PlayBallUiContext;

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
    fn set_state(&mut self, _state: &GameState) {}
    fn set_context(&mut self, _ctx: &PlayBallUiContext) {}
}
