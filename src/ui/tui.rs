use std::io;

use crossterm::event::KeyEventKind;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::ui::Ui;
use crate::ui::events::UiEvent;

/// Minimal TUI implementation:
/// - scrollable output pane (above)
/// - input prompt anchored at the bottom
///
/// This is a *structural* UI layer: the engine remains unchanged.
/// Future steps can add richer styling, command history, better editing, etc.
pub struct TuiUi {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    log: Vec<String>,
    input: String,
    scroll: u16, // number of lines scrolled from the top
}

impl TuiUi {
    pub fn init_terminal() -> io::Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        Ok(())
    }

    pub fn restore_terminal() -> io::Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            log: Vec::new(),
            input: String::new(),
            scroll: 0,
        })
    }

    fn push_line(&mut self, s: String) {
        self.log.push(s);
        // auto-scroll to bottom (follow mode)
        self.scroll_to_bottom();
    }

    fn scroll_to_bottom(&mut self) {
        // scroll is measured from the top; easiest is to clamp during render.
        // Here we just keep it large; render will clamp to max.
        self.scroll = u16::MAX;
    }

    fn scroll_up(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    fn scroll_down(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_add(n);
    }

    fn render(&mut self, prompt: &str) -> io::Result<()> {
        let log_len = self.log.len() as u16;

        self.terminal.draw(|f| {
            let size = f.area();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                .split(size);

            // Build log text
            let mut text = Text::default();
            for line in &self.log {
                text.lines.push(Line::from(line.as_str()));
            }

            // Compute max scroll based on viewport height
            let viewport_h = chunks[0].height.saturating_sub(2); // account for borders
            let max_scroll = log_len.saturating_sub(viewport_h);

            let mut scroll = self.scroll;
            scroll = scroll.min(max_scroll);

            let log_widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Log"))
                .wrap(Wrap { trim: false })
                .scroll((scroll, 0));

            f.render_widget(log_widget, chunks[0]);

            // Input (prompt + current input)
            let input_line = format!("{prompt}{}", self.input);

            let input_widget = Paragraph::new(input_line)
                .block(Block::default().borders(Borders::ALL).title("Command"))
                .style(Style::default());

            f.render_widget(input_widget, chunks[1]);

            // Place cursor at end of input
            let x =
                chunks[1].x + 1 + prompt.chars().count() as u16 + self.input.chars().count() as u16;
            let y = chunks[1].y + 1;
            f.set_cursor_position((x.min(chunks[1].x + chunks[1].width.saturating_sub(2)), y));
        })?;

        Ok(())
    }
}

impl Drop for TuiUi {
    fn drop(&mut self) {
        // Best-effort cleanup (cannot return errors in Drop)
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

impl Ui for TuiUi {
    fn emit(&mut self, event: UiEvent) {
        match event {
            UiEvent::Line(s) => self.push_line(s),
            UiEvent::Success(s) => self.push_line(format!("✅ {s}")),
            UiEvent::Error(s) => self.push_line(format!("❌ {s}")),
        }
    }

    fn read_command_line(&mut self, prompt: &str) -> Option<String> {
        self.input.clear();

        loop {
            if self.render(prompt).is_err() {
                return None;
            }

            let ev = match event::read() {
                Ok(ev) => ev,
                Err(_) => return None,
            };

            match ev {
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    ..
                }) => {
                    // IMPORTANT: avoid duplicate input on Repeat/Release
                    if kind != KeyEventKind::Press {
                        continue;
                    }

                    // Global exits
                    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                        return None;
                    }

                    match code {
                        KeyCode::Enter => {
                            let line = self.input.trim().to_string();
                            return Some(line);
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                        }
                        KeyCode::Char(c) => {
                            // Ignore control characters; allow typical printable chars
                            if !modifiers.contains(KeyModifiers::CONTROL)
                                && !modifiers.contains(KeyModifiers::ALT)
                            {
                                self.input.push(c);
                            }
                        }
                        KeyCode::Esc => {
                            self.input.clear();
                        }
                        KeyCode::Up => self.scroll_up(1),
                        KeyCode::Down => self.scroll_down(1),
                        KeyCode::PageUp => self.scroll_up(10),
                        KeyCode::PageDown => self.scroll_down(10),
                        KeyCode::Home => self.scroll = 0,
                        KeyCode::End => self.scroll_to_bottom(),
                        _ => {}
                    }
                }
                Event::Resize(_, _) => {
                    // re-render next tick
                }
                _ => {}
            }
        }
    }
}
