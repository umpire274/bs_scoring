use std::io::{self, Write};

use crate::ui::Ui;
use crate::ui::events::UiEvent;

/// Simple stdin/stdout UI.
///
/// This is intentionally minimal so that we can later swap it with a ratatui-based UI
/// that keeps the prompt on the last line and scrolls the log above it.
pub struct CliUi;

impl CliUi {
    pub fn new() -> Self {
        Self
    }
}

impl Ui for CliUi {
    fn emit(&mut self, event: UiEvent) {
        match event {
            UiEvent::Line(s) => println!("{s}"),
            UiEvent::Success(s) => println!("{s}"),
            UiEvent::Error(s) => println!("❌ {s}"),
        }
    }

    fn read_command_line(&mut self, prompt: &str) -> Option<String> {
        print!("{prompt}");
        io::stdout().flush().ok()?;

        let mut line = String::new();
        io::stdin().read_line(&mut line).ok()?;
        Some(line)
    }
}

impl Default for CliUi {
    fn default() -> Self {
        Self::new()
    }
}
