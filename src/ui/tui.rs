use std::io;

use crate::models::game_state::GameState;
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

/// Which panel currently receives scroll input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Log,
    Help,
    Command,
}

/// Minimal TUI implementation:
/// - scrollable output pane (left)
/// - scoreboard + help on the right
/// - one-line command input at the bottom
/// - command history recall with Up/Down when Command has focus
pub struct TuiUi {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    log: Vec<String>,
    input: String,
    command_history: Vec<String>,
    history_index: Option<usize>,
    scroll: u16, // log scroll from top
    help_scroll: u16,
    focus: Focus,
    state: Option<GameState>,
    ctx: Option<PlayBallUiContext>,
}

#[derive(Debug, Clone)]
struct ScoreboardViewData {
    inning: u32,
    half_sym: &'static str,
    outs: u8,

    away_score: u16,
    home_score: u16,
    away_hits: u16,
    home_hits: u16,
    away_errors: u16,
    home_errors: u16,
    away_innings: Vec<u16>,
    home_innings: Vec<u16>,

    pub batter_left: String,
    pub batter_right: String,

    pitcher_first_name: String,
    pitcher_last_name: String,

    count: String,
    on_1b: bool,
    on_2b: bool,
    on_3b: bool,
}

#[derive(Debug, Clone, Copy)]
struct RheTotals {
    runs: u16,
    hits: u16,
    errors: u16,
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
            command_history: Vec::new(),
            history_index: None,
            scroll: 0,
            help_scroll: 0,
            focus: Focus::Log,
            state: None,
            ctx: None,
        })
    }

    fn push_line(&mut self, s: String) {
        self.log.push(s);
        self.scroll_to_bottom();
    }

    fn scroll_to_bottom(&mut self) {
        self.scroll = u16::MAX;
    }

    fn scroll_up(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    fn scroll_down(&mut self, n: u16) {
        self.scroll = self.scroll.saturating_add(n);
    }

    fn log_line_count(&self) -> u16 {
        let mut total = 0u16;

        for entry in &self.log {
            let lines = entry.lines().count().max(1) as u16;
            total = total.saturating_add(lines);
        }

        total
    }

    fn clamp_scroll_to_viewport(&mut self) {
        let size = match self.terminal.size() {
            Ok(size) => size,
            Err(_) => return,
        };

        let outer = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Length(1),
                    Constraint::Length(3),
                ]
                .as_ref(),
            )
            .split(Rect::from(size));

        let top = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(20), Constraint::Length(50)].as_ref())
            .split(outer[0]);

        let log_area = top[0];
        let right_pane = top[1];

        let viewport_h = log_area.height.saturating_sub(2);
        let log_len = self.log_line_count();
        let max_scroll = log_len.saturating_sub(viewport_h);
        self.scroll = self.scroll.min(max_scroll);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(13), Constraint::Min(5)].as_ref())
            .split(right_pane);

        let help_viewport_h = right[1].height.saturating_sub(2);
        let help_lines = Self::help_line_count() as u16;
        let max_help_scroll = help_lines.saturating_sub(help_viewport_h);
        self.help_scroll = self.help_scroll.min(max_help_scroll);
    }

    pub fn set_state(&mut self, state: GameState) {
        self.state = Some(state);
    }

    fn recall_previous_command(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        let next_index = match self.history_index {
            None => self.command_history.len().checked_sub(1),
            Some(i) => i.checked_sub(1),
        };

        if let Some(i) = next_index {
            self.history_index = Some(i);
            self.input = self.command_history[i].clone();
        }
    }

    fn recall_next_command(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        match self.history_index {
            Some(i) if i + 1 < self.command_history.len() => {
                let next = i + 1;
                self.history_index = Some(next);
                self.input = self.command_history[next].clone();
            }
            Some(_) => {
                self.history_index = None;
                self.input.clear();
            }
            None => {}
        }
    }

    fn fit_two_columns(left: &str, right: &str, width: usize) -> String {
        if width == 0 {
            return String::new();
        }

        let right_w = Self::display_width(right);
        if right_w >= width {
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

        let target = max_width - 1;
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

    fn visible_inning_range(total_innings: usize, width: usize) -> (usize, usize) {
        let total = total_innings.max(9);

        let reserved = 17usize;
        let per_inning = 3usize;

        let available = width.saturating_sub(reserved);
        let max_visible = (available / per_inning).max(1);

        if total <= max_visible {
            (1, total)
        } else {
            (total - max_visible + 1, total)
        }
    }

    fn inning_value(innings: &[u16], inning_no: usize) -> u16 {
        innings.get(inning_no - 1).copied().unwrap_or(0)
    }

    fn render_linescore_header(start_inning: usize, end_inning: usize, width: usize) -> String {
        let innings = (start_inning..=end_inning)
            .map(|n| format!("{:>2}", n))
            .collect::<Vec<_>>()
            .join(" ");

        let left = format!("      {}", innings);
        let right = format!("| {:>3} {:>3} {:>3} ", "R", "H", "E");

        Self::fit_two_columns(&left, &right, width)
    }

    fn render_linescore_row(
        team_abbr: &str,
        innings: &[u16],
        start_inning: usize,
        end_inning: usize,
        totals: RheTotals,
        width: usize,
    ) -> String {
        let inning_cells = (start_inning..=end_inning)
            .map(|inning_no| format!("{:>2}", Self::inning_value(innings, inning_no)))
            .collect::<Vec<_>>()
            .join(" ");

        let left = format!(" {:<3}  {}", team_abbr, inning_cells);

        let rhe = format!(
            "| {:>3} {:>3} {:>3} ",
            totals.runs, totals.hits, totals.errors
        );

        Self::fit_two_columns(&left, &rhe, width)
    }

    fn render_base_diamond(
        width: usize,
        on_1b: bool,
        on_2b: bool,
        on_3b: bool,
    ) -> (String, String) {
        let b1 = if on_1b { '◆' } else { '◇' };
        let b2 = if on_2b { '◆' } else { '◇' };
        let b3 = if on_3b { '◆' } else { '◇' };

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

    fn format_player_name_for_scoreboard(first: &str, last: &str, max_len: usize) -> String {
        let full = format!("P. {first} {last}");
        if full.chars().count() <= max_len {
            return full;
        }

        let initial = first.chars().next().unwrap_or('?');
        let abbr = format!("P. {}. {last}", initial.to_uppercase());
        if abbr.chars().count() <= max_len {
            return abbr;
        }

        let last_only = format!("P. {last}");
        if last_only.chars().count() <= max_len {
            return last_only;
        }

        Self::ellipsize(&last_only, max_len)
    }

    fn scoreboard_view_data(state: Option<&GameState>) -> ScoreboardViewData {
        if let Some(s) = state {
            let (batter_left, batter_right) = match (
                s.current_batter_order,
                s.current_batter_first_name.as_deref(),
                s.current_batter_last_name.as_deref(),
                s.current_batter_jersey_no,
                s.current_batter_position,
            ) {
                (Some(order), Some(first), Some(last), Some(jersey), Some(pos)) => (
                    format!("{}. {} {}", order, first, last),
                    format!("(#{} {})", jersey, pos),
                ),
                _ => ("-".to_string(), "".to_string()),
            };

            let count = format!("{}-{}", s.pitch_count.balls, s.pitch_count.strikes);

            ScoreboardViewData {
                inning: s.inning,
                half_sym: s.half_symbol(),
                outs: s.outs,

                away_score: s.score.away,
                home_score: s.score.home,
                away_hits: s.score.away_hits,
                home_hits: s.score.home_hits,
                away_errors: 0,
                home_errors: 0,
                away_innings: s.score.away_innings.clone(),
                home_innings: s.score.home_innings.clone(),

                batter_left,
                batter_right,

                pitcher_first_name: s
                    .current_pitcher_first_name
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                pitcher_last_name: s
                    .current_pitcher_last_name
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),

                count,
                on_1b: s.on_1b.is_some(),
                on_2b: s.on_2b.is_some(),
                on_3b: s.on_3b.is_some(),
            }
        } else {
            ScoreboardViewData {
                inning: 1,
                half_sym: "↑",
                outs: 0,

                away_score: 0,
                home_score: 0,
                away_hits: 0,
                home_hits: 0,
                away_errors: 0,
                home_errors: 0,
                away_innings: Vec::new(),
                home_innings: Vec::new(),

                batter_left: "-".to_string(),
                batter_right: "".to_string(),

                pitcher_first_name: "-".to_string(),
                pitcher_last_name: "-".to_string(),

                count: "0-0".to_string(),
                on_1b: false,
                on_2b: false,
                on_3b: false,
            }
        }
    }

    fn build_linescore_lines(
        ctx: Option<&PlayBallUiContext>,
        data: &ScoreboardViewData,
        width: usize,
    ) -> (String, String, String) {
        let (away, home) = match ctx {
            Some(c) => (c.away_abbr.as_str(), c.home_abbr.as_str()),
            None => ("AWY", "HOM"),
        };

        let total_innings = data
            .away_innings
            .len()
            .max(data.home_innings.len())
            .max(data.inning as usize)
            .max(9);

        let (start_inning, end_inning) = Self::visible_inning_range(total_innings, width);

        let header = Self::render_linescore_header(start_inning, end_inning, width);

        let away_line = Self::render_linescore_row(
            away,
            &data.away_innings,
            start_inning,
            end_inning,
            RheTotals {
                runs: data.away_score,
                hits: data.away_hits,
                errors: data.away_errors,
            },
            width,
        );

        let home_line = Self::render_linescore_row(
            home,
            &data.home_innings,
            start_inning,
            end_inning,
            RheTotals {
                runs: data.home_score,
                hits: data.home_hits,
                errors: data.home_errors,
            },
            width,
        );

        (header, away_line, home_line)
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

        let w = inner.width as usize;
        let data = Self::scoreboard_view_data(state);
        let (header, line_away, line_home) = Self::build_linescore_lines(ctx, &data, w);
        let (d_top, d_bot) = Self::render_base_diamond(w, data.on_1b, data.on_2b, data.on_3b);

        let outs_str = Self::outs_dots(data.outs);
        let status = format!(
            "{}{}  {}  {}",
            data.inning, data.half_sym, data.count, outs_str
        );
        let status_line = Self::pad_right_fit(Self::center_text(&status, w).as_str(), w);

        let batter_line =
            Self::fit_two_columns(data.batter_left.as_str(), data.batter_right.as_str(), w);

        let (pitches, strikes, balls) = if let Some(s) = state {
            if let Some(pid) = s.current_pitcher_id {
                if let Some(stats) = s.pitcher_stats.get(&pid) {
                    (stats.balls + stats.strikes, stats.strikes, stats.balls)
                } else {
                    (0, 0, 0)
                }
            } else {
                (0, 0, 0)
            }
        } else {
            (0, 0, 0)
        };

        let pitcher_right = format!("(P {}: {}-{})", pitches, strikes, balls);
        let max_left = w.saturating_sub(Self::display_width(&pitcher_right) + 1);

        let pitcher_left = Self::format_player_name_for_scoreboard(
            data.pitcher_first_name.as_str(),
            data.pitcher_last_name.as_str(),
            max_left,
        );

        let pitcher_line = Self::fit_two_columns(&pitcher_left, &pitcher_right, w);

        let lines = vec![
            Line::from(Self::pad_right_fit(header.as_str(), w)),
            Line::from(Self::pad_right_fit(line_away.as_str(), w)),
            Line::from(Self::pad_right_fit(line_home.as_str(), w)),
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

    fn help_line_count() -> usize {
        Self::help_lines().len()
    }

    fn help_lines() -> Vec<Line<'static>> {
        vec![
            Line::from("Pitch commands"),
            Line::from("  b          Ball"),
            Line::from("  k          Called strike"),
            Line::from("  s          Swinging strike"),
            Line::from("  f          Foul"),
            Line::from("  fl         Foul bunt"),
            Line::from(""),
            Line::from("Hit commands"),
            Line::from("  h  [zone]  Single"),
            Line::from("  2h [zone]  Double"),
            Line::from("  3h [zone]  Triple"),
            Line::from("  hr [zone]  Home run"),
            Line::from(""),
            Line::from("Hit zones"),
            Line::from("  LL LF LC CF RC RF RL"),
            Line::from("  GLL LS MI RS GRL"),
            Line::from(""),
            Line::from("Steal commands"),
            Line::from("  <n> st <base>      Steal (1 st 2b, 3 st sc)"),
            Line::from(""),
            Line::from("Out commands"),
            Line::from("  <n> 63             Ground out"),
            Line::from("  <n> 5              Unassisted out"),
            Line::from("  <n> f8             Fly out"),
            Line::from("  <n> ff2            Foul fly out"),
            Line::from("  <n> l6             Line out"),
            Line::from("  <n> if4            Infield fly"),
            Line::from(""),
            Line::from("Defensive play commands"),
            Line::from("  63                 Batter implicit ground out"),
            Line::from("  5                  Batter implicit unassisted out"),
            Line::from("  f9                 Batter implicit fly out"),
            Line::from("  l6                 Batter implicit line out"),
            Line::from("  if4                Batter implicit infield fly"),
            Line::from("  <n> o6 1b          Fielder's choice"),
            Line::from("  9 64, 1 o6 1b      Multi-command defensive play"),
            Line::from(""),
            Line::from("Notes"),
            Line::from("  - Commands are case-insensitive."),
            Line::from("  - Fielder's choice requires an explicit base."),
            Line::from("  - Infield fly is valid only with <2 outs and runners on 1B+2B."),
        ]
    }

    fn render_help(f: &mut Frame, area: Rect, scroll: u16, focused: bool) {
        let title = if focused { "Help ►" } else { "Help" };
        let block = Block::default().borders(Borders::ALL).title(title);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let p = Paragraph::new(Text::from(Self::help_lines()))
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0));

        f.render_widget(p, inner);
    }

    fn render_command(f: &mut Frame, area: Rect, prompt: &str, input: &str, focused: bool) {
        let title = if focused { "Command ►" } else { "Command" };
        let block = Block::default().borders(Borders::ALL).title(title);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let line = Line::from(format!("{prompt}{input}"));
        let p = Paragraph::new(Text::from(vec![line]))
            .wrap(Wrap { trim: false })
            .style(Style::default());

        f.render_widget(p, inner);
    }

    fn render_shortcuts(f: &mut Frame, area: Rect, focus: Focus) {
        let focus_label = match focus {
            Focus::Log => "Log",
            Focus::Help => "Help",
            Focus::Command => "Command",
        };

        let bar = format!(
            " - focus on:{focus_label} - Tab:change focus ↑↓:scroll/history  PgUp/Dn:page  Home/End:top/bot"
        );
        let p = Paragraph::new(bar).style(Style::default());
        f.render_widget(p, area);
    }

    fn render(&mut self, prompt: &str) -> io::Result<()> {
        self.clamp_scroll_to_viewport();

        let log = self.log.clone();
        let input = self.input.clone();
        let scroll = self.scroll;
        let help_scroll = self.help_scroll;
        let focus = self.focus;
        let state = self.state.clone();
        let ctx = self.ctx.clone();
        let prompt = prompt.to_string();

        self.terminal.draw(move |f| {
            let size = f.area();

            let outer = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(1),
                        Constraint::Length(1),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let top = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(20), Constraint::Length(50)].as_ref())
                .split(outer[0]);

            let log_area = top[0];
            let right_pane = top[1];
            let shortcuts_area = outer[1];
            let command_area = outer[2];

            let right = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(13), Constraint::Min(5)].as_ref())
                .split(right_pane);

            let scoreboard_area = right[0];
            let help_area = right[1];

            let mut text = Text::default();
            for line in &log {
                text.lines.push(Line::from(line.as_str()));
            }

            let log_title = if focus == Focus::Log {
                "Log ►"
            } else {
                "Log"
            };

            let log_widget = Paragraph::new(text)
                .block(Block::default().borders(Borders::ALL).title(log_title))
                .wrap(Wrap { trim: false })
                .scroll((scroll, 0));

            f.render_widget(log_widget, log_area);
            Self::render_scoreboard(ctx.as_ref(), state.as_ref(), f, scoreboard_area);
            Self::render_help(f, help_area, help_scroll, focus == Focus::Help);
            Self::render_shortcuts(f, shortcuts_area, focus);
            Self::render_command(f, command_area, &prompt, &input, focus == Focus::Command);

            let inner = Block::default().borders(Borders::ALL).inner(command_area);
            let cursor_y = inner.y;
            let cursor_x = inner.x + prompt.chars().count() as u16 + input.chars().count() as u16;

            f.set_cursor_position((
                cursor_x.min(inner.x + inner.width.saturating_sub(1)),
                cursor_y,
            ));
        })?;

        Ok(())
    }
}

impl Drop for TuiUi {
    fn drop(&mut self) {
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
        self.history_index = None;

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
                    if kind != KeyEventKind::Press {
                        continue;
                    }

                    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
                        return None;
                    }

                    match code {
                        KeyCode::Enter => {
                            let line = self.input.trim().to_string();
                            if !line.is_empty() {
                                self.command_history.push(line.clone());
                            }
                            self.history_index = None;
                            return Some(line);
                        }

                        KeyCode::Backspace => {
                            self.input.pop();
                            self.history_index = None;
                        }

                        KeyCode::Char(c)
                            if !modifiers.contains(KeyModifiers::CONTROL)
                                && !modifiers.contains(KeyModifiers::ALT) =>
                        {
                            self.input.push(c);
                            self.history_index = None;
                        }

                        KeyCode::Esc => {
                            self.input.clear();
                            self.history_index = None;
                        }

                        KeyCode::Tab => {
                            self.focus = match self.focus {
                                Focus::Log => Focus::Help,
                                Focus::Help => Focus::Command,
                                Focus::Command => Focus::Log,
                            };
                        }

                        KeyCode::Up => match self.focus {
                            Focus::Log => self.scroll_up(1),
                            Focus::Help => {
                                self.help_scroll = self.help_scroll.saturating_sub(1);
                            }
                            Focus::Command => self.recall_previous_command(),
                        },

                        KeyCode::Down => match self.focus {
                            Focus::Log => self.scroll_down(1),
                            Focus::Help => {
                                self.help_scroll = self.help_scroll.saturating_add(1);
                            }
                            Focus::Command => self.recall_next_command(),
                        },

                        KeyCode::PageUp => match self.focus {
                            Focus::Log => self.scroll_up(10),
                            Focus::Help => {
                                self.help_scroll = self.help_scroll.saturating_sub(10);
                            }
                            Focus::Command => {}
                        },

                        KeyCode::PageDown => match self.focus {
                            Focus::Log => self.scroll_down(10),
                            Focus::Help => {
                                self.help_scroll = self.help_scroll.saturating_add(10);
                            }
                            Focus::Command => {}
                        },

                        KeyCode::Home => match self.focus {
                            Focus::Log => self.scroll = 0,
                            Focus::Help => self.help_scroll = 0,
                            Focus::Command => {
                                if !self.command_history.is_empty() {
                                    self.history_index = Some(0);
                                    self.input = self.command_history[0].clone();
                                }
                            }
                        },

                        KeyCode::End => match self.focus {
                            Focus::Log => self.scroll_to_bottom(),
                            Focus::Help => self.help_scroll = u16::MAX,
                            Focus::Command => {
                                self.history_index = None;
                                self.input.clear();
                            }
                        },

                        _ => {}
                    }
                }

                Event::Resize(_, _) => {}
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
