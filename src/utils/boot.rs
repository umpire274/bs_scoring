use anyhow::Result;
use crossterm::style::{Color, Stylize};
use figlet_rs::FIGlet;
use std::io::Write;
use std::time::Duration;
use std::{io, thread};

const DOOM: &str = include_str!("../../res/doom.flf");

#[derive(Debug, Clone, Copy)]
pub enum DbBootStatus {
    ReadyExisting,
    ReadyNew,
    Ready,
    Created,
    Migrated(i64),
}

impl DbBootStatus {
    pub fn label(&self) -> String {
        match self {
            DbBootStatus::ReadyExisting => "✅ Existing database ready".to_string(),
            DbBootStatus::ReadyNew => "✅ New database ready".to_string(),
            DbBootStatus::Ready => "✅ Database ready".to_string(),
            DbBootStatus::Created => "🆕 Database created".to_string(),
            DbBootStatus::Migrated(n) => format!("🔧 Database migrated ({n} migration(s))"),
        }
    }
}

/// One-line animated boot step with a smooth Unicode spinner.
/// Prints something like:
/// [1/3] Opening database   ⠋ ... ⠙ ... ⠹ ... ✔
///
/// `work` should do the actual step and return Ok/Err.
pub fn boot_step<F>(step: usize, total: usize, label: &str, mut work: F) -> Result<()>
where
    F: FnMut() -> Result<()>,
{
    const FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let label_width = 24usize;

    let mut stdout = io::stdout();

    // mini-animazione
    for frame in FRAMES {
        print!(
            "\r[{}/{}] {:<width$} {}",
            step,
            total,
            label,
            frame.with(Color::DarkGrey),
            width = label_width
        );
        stdout.flush().ok();
        thread::sleep(Duration::from_millis(55));
    }

    // esegue davvero il lavoro (qui puoi fallire con ?)
    let res = work();

    match res {
        Ok(()) => {
            let done = "✔".with(Color::Green);
            print!(
                "\r[{}/{}] {:<width$} {}\n",
                step,
                total,
                label,
                done,
                width = label_width
            );
            stdout.flush().ok();
            Ok(())
        }
        Err(e) => {
            let fail = "✖".with(Color::Red);
            print!(
                "\r[{}/{}] {:<width$} {}\n",
                step,
                total,
                label,
                fail,
                width = label_width
            );
            println!("{e:#}");
            stdout.flush().ok();
            Err(e)
        }
    }
}

pub fn boot_screen_header() {
    println!();

    if let Ok(font) = FIGlet::from_content(DOOM)
        && let Some(fig) = font.convert("BS Scoring")
    {
        println!("{fig}");
    }

    println!("BS Scoring v{}", env!("CARGO_PKG_VERSION"));
    println!();
}

pub fn boot_screen_footer(db_path: &str, status: &DbBootStatus) {
    println!();
    println!("Database: {db_path}");
    println!("Status: {}", status.label());
}
