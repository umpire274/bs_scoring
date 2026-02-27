use std::io;

use crate::models::play_ball::GameState;
use crate::ui::events::UiEvent;
use crate::ui::{PlayBallUiContext, Ui};
use crossterm::event::KeyEventKind;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::layout::Rect;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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
    state: Option<GameState>,
    ctx: Option<PlayBallUiContext>,
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
            state: None,
            ctx: None,
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

    pub fn set_state(&mut self, state: GameState) {
        self.state = Some(state);
    }

    fn fit_two_columns(left: &str, right: &str, width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let right_w = Self::display_width(right);
        if right_w >= width {
            // caso estremo: tronco right a width
            return Self::ellipsize(right, width);
        }

        let gap = 1usize;
        let left_max = width.saturating_sub(right_w + gap);

        let left_fitted = Self::ellipsize(left, left_max);
        let left_w = Self::display_width(&left_fitted);

        let spaces = width.saturating_sub(right_w + left_w);
        let mut out = String::with_capacity(width * 4);
        out.push_str(&left_fitted);
        for _ in 0..spaces {
            out.push(' ');
        }
        out.push_str(right);
        out
    }

    fn ellipsize(s: &str, max_width: usize) -> String {
        if max_width == 0 {
            return String::new();
        }
        if Self::display_width(s) <= max_width {
            return s.to_string();
        }
        if max_width == 1 {
            return "…".to_string();
        }

        let target = max_width - 1; // spazio per '…'
        let mut out = String::with_capacity(s.len().min(max_width * 4));
        let mut w = 0usize;

        for ch in s.chars() {
            let cw = UnicodeWidthChar::width(ch).unwrap_or(0);
            if w + cw > target {
                break;
            }
            out.push(ch);
            w += cw;
        }

        out.push('…');
        out
    }

    fn display_width(s: &str) -> usize {
        UnicodeWidthStr::width(s)
    }

    fn pad_right(s: &str, width: usize) -> String {
        let current = Self::display_width(s);

        if current >= width {
            return s.to_string();
        }

        let mut out = String::with_capacity(width * 4);
        out.push_str(s);

        for _ in 0..(width - current) {
            out.push(' ');
        }

        out
    }

    fn pad_right_fit(s: &str, width: usize) -> String {
        let fitted = Self::ellipsize(s, width);
        Self::pad_right(&fitted, width)
    }

    fn center_text(s: &str, width: usize) -> String {
        let w = Self::display_width(s);
        if w >= width {
            return s.to_string();
        }

        let left = (width - w) / 2;
        let right = width - w - left;

        let mut out = String::with_capacity(width * 4);
        for _ in 0..left {
            out.push(' ');
        }
        out.push_str(s);
        for _ in 0..right {
            out.push(' ');
        }
        out
    }

    /// Returns (top_line, bottom_line), both padded/centered to `width`.
    fn render_base_diamond(
        width: usize,
        on_1b: bool,
        on_2b: bool,
        on_3b: bool,
    ) -> (String, String) {
        let b1 = if on_1b { '◆' } else { '◇' };
        let b2 = if on_2b { '◆' } else { '◇' };
        let b3 = if on_3b { '◆' } else { '◇' };

        // NO leading spaces here
        let top_raw = format!("{b2}");
        let bottom_raw = format!("{b3}   {b1}");

        let top = Self::pad_right(&Self::center_text(&top_raw, width), width);
        let bottom = Self::pad_right(&Self::center_text(&bottom_raw, width), width);

        (top, bottom)
    }

    fn outs_dots(outs: u8) -> String {
        let o1 = if outs >= 1 { '●' } else { '○' };
        let o2 = if outs >= 2 { '●' } else { '○' };

        format!("OUT {} {}", o1, o2)
    }

    fn format_player_name_for_scoreboard(
        jersey: i32,
        first: &str,
        last: &str,
        max_len: usize,
    ) -> String {
        let full = format!("#{jersey} {first} {last}");
        if full.chars().count() <= max_len {
            return full;
        }

        let initial = first.chars().next().unwrap_or('?');
        let abbr = format!("#{jersey} {}. {last}", initial.to_uppercase());
        if abbr.chars().count() <= max_len {
            return abbr;
        }

        let last_only = format!("#{jersey} {last}");
        if last_only.chars().count() <= max_len {
            return last_only;
        }

        // ultima spiaggia: tronca con ellissi
        Self::ellipsize(&last_only, max_len)
    }

    fn render_scoreboard(
        ctx: Option<&PlayBallUiContext>,
        state: Option<&GameState>,
        f: &mut Frame,
        area: Rect,
    ) {
        let block = Block::default().borders(Borders::ALL).title("Scoreboard");
        let inner = block.inner(area);
        f.render_widget(block, area);

        // 34 colonne utili con bordo su pannello da 36
        let w = inner.width as usize;

        // ----- extraction -----
        let (
            inning,
            half_sym,
            outs,
            away_score,
            home_score,
            batter,
            p_jersey_no,
            p_first_name,
            p_last_name,
            count,
            on_1b,
            on_2b,
            on_3b,
            current_pitch_count,
        ) = if let Some(s) = state {
            let batter = match (
                s.current_batter_jersey_no,
                s.current_batter_first_name.as_deref(),
                s.current_batter_last_name.as_deref(),
            ) {
                (Some(j), Some(first), Some(last)) => format!("#{j}  {first} {last}"),
                _ => "-".to_string(),
            };

            let count = format!("{}-{}", s.pitch_count.balls, s.pitch_count.strikes);

            (
                s.inning,
                s.half_symbol(), // ↑ / ↓
                s.outs,
                s.score.away,
                s.score.home,
                batter,
                s.current_pitcher_jersey_no.unwrap_or(0),
                s.current_pitcher_first_name.as_deref().unwrap_or("-"),
                s.current_pitcher_last_name.as_deref().unwrap_or("-"),
                count,
                s.on_1b,
                s.on_2b,
                s.on_3b,
                s.current_pitch_count,
            )
        } else {
            (
                1,
                "↑",
                0,
                0,
                0,
                "-".to_string(),
                0,
                "-",
                "-",
                "0-0".to_string(),
                false,
                false,
                false,
                0,
            )
        };

        // ✅ centered diamond based on display width
        let (d_top, d_bot) = Self::render_base_diamond(w, on_1b, on_2b, on_3b);

        // Outs dots
        let outs_str = Self::outs_dots(outs);

        let status = format!("{}{}  {}  {}", half_sym, inning, count, outs_str);
        let status_line = Self::pad_right_fit(Self::center_text(&status, w).as_str(), w);

        // Teams abbreviations
        let (away, home) = match ctx {
            Some(c) => (c.away_abbr.as_str(), c.home_abbr.as_str()),
            None => ("AWY", "HOM"),
        };

        // ----- lines -----
        let header = "      1 2 3 4 5 6 7 8 9 10  R H E";
        let line_mod = format!(" {away}  0 0 0 0 0 0 0 0 0 0   {away_score} 0 0");
        let line_bol = format!(" {home}  0 0 0 0 0 0 0 0 0 0   {home_score} 0 0");

        let batter_line = Self::pad_right_fit(batter.as_str(), w);

        let right = format!("(P {:>3})", current_pitch_count);
        let max_left = w.saturating_sub(Self::display_width(&right) + 1); // +1 gap
        let pitcher_left = Self::format_player_name_for_scoreboard(
            p_jersey_no,
            p_first_name,
            p_last_name,
            max_left,
        );
        let pitcher_line = Self::fit_two_columns(&pitcher_left, &right, w);

        let lines = vec![
            Line::from(Self::pad_right_fit(header, w)),
            Line::from(Self::pad_right_fit(line_mod.as_str(), w)),
            Line::from(Self::pad_right_fit(line_bol.as_str(), w)),
            Line::from(Self::pad_right_fit("", w)),
            Line::from(d_top),
            Line::from(d_bot),
            Line::from(Self::pad_right_fit("", w)),
            Line::from(Self::pad_right_fit(status_line.as_str(), w)),
            Line::from(Self::pad_right_fit("", w)),
            Line::from(batter_line),
            Line::from(pitcher_line),
        ];

        let p = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
        f.render_widget(p, inner);
    }
    fn render(&mut self, prompt: &str) -> io::Result<()> {
        let log_len = self.log.len() as u16;

        // Copia i pezzi che servono dentro la closure per evitare di catturare `&self`.
        let log = self.log.clone();
        let input = self.input.clone();
        let scroll_in = self.scroll;
        let state = self.state.clone();
        let ctx = self.ctx.clone();
        let prompt = prompt.to_string();

        self.terminal.draw(move |f| {
            let size = f.area();

            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                .split(size);

            let top = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(20), Constraint::Max(36)].as_ref())
                .split(outer[0]);

            let log_area = top[0];
            let scoreboard_area = top[1];
            let command_area = outer[1];

            // Build log text
            let mut text = Text::default();
            for line in &log {
                text.lines.push(Line::from(line.as_str()));
            }

            // Compute max scroll based on viewport height
            let viewport_h = log_area.height.saturating_sub(2); // account for borders
            let max_scroll = log_len.saturating_sub(viewport_h);

            let mut scroll = scroll_in;
            scroll = scroll.min(max_scroll);

            let log_widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title("Log"))
                .wrap(Wrap { trim: false })
                .scroll((scroll, 0));

            f.render_widget(log_widget, log_area);
            Self::render_scoreboard(ctx.as_ref(), state.as_ref(), f, scoreboard_area);

            // Input (prompt + current input)
            let input_line = format!("{prompt}{input}");

            let input_widget = Paragraph::new(input_line)
                .block(Block::default().borders(Borders::ALL).title("Command"))
                .style(Style::default());

            f.render_widget(input_widget, command_area);

            // Place cursor at end of input
            let x =
                command_area.x + 1 + prompt.chars().count() as u16 + input.chars().count() as u16;
            let y = command_area.y + 1;
            f.set_cursor_position((
                x.min(command_area.x + command_area.width.saturating_sub(2)),
                y,
            ));
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

    fn set_state(&mut self, state: &GameState) {
        self.state = Some(state.clone());
    }

    fn set_context(&mut self, ctx: &PlayBallUiContext) {
        self.ctx = Some(ctx.clone());
    }
}
