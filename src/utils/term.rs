use crate::models::player_traits::{BatSide, ThrowHand};
use std::io;
use std::io::Write;

/// Clear the screen (works on most terminals)
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Read a numeric choice from user
pub(crate) fn read_choice() -> u32 {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().parse().unwrap_or(0)
}

/// Read a string input from user
pub fn read_string(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

/// Read an optional string (can be empty)
pub fn read_optional_string(prompt: &str) -> Option<String> {
    let input = read_string(prompt);
    if input.is_empty() { None } else { Some(input) }
}

/// Read an integer
pub fn read_i32(prompt: &str) -> Option<i32> {
    let input = read_string(prompt);
    input.parse().ok()
}

/// Read an i64 (returns None if empty or invalid — backward compatible)
pub fn read_i64(prompt: &str) -> Option<i64> {
    let input = read_string(prompt);
    input.parse().ok()
}

/// Read an i64, retrying until a valid number is entered.
pub fn read_i64_required(prompt: &str) -> i64 {
    loop {
        let input = read_string(prompt);
        match input.parse::<i64>() {
            Ok(n) => return n,
            Err(_) => println!("  ⚠️  Please enter a valid number."),
        }
    }
}

/// Read a string with a default (ENTER returns the default)
pub fn read_string_with_default(prompt: &str, default: &str) -> String {
    let input = read_string(prompt);
    if input.is_empty() {
        default.to_string()
    } else {
        input
    }
}

/// Read an optional string with a default (ENTER keeps current value)
pub fn read_optional_string_with_default(prompt: &str, current: Option<&str>) -> Option<String> {
    let input = read_string(prompt);
    if input.is_empty() {
        current.map(|s| s.to_string())
    } else {
        Some(input)
    }
}

/// Wait for user to press enter
pub fn wait_for_enter() {
    println!("\nPress ENTER to continue...");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
}

/// Display a success message
pub fn show_success(message: &str) {
    println!("\n✅ {}", message);
    wait_for_enter();
}

pub fn show_success_no_wait_for_enter(message: &str) {
    println!("\n✅ {}", message);
}

/// Display an error message
pub fn show_error(message: &str) {
    println!("\n❌ Error: {}", message);
    wait_for_enter();
}

/// Confirm action
pub fn confirm(message: &str) -> bool {
    print!("{} (y/n): ", message);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase() == "y"
}

/// Display a header
pub fn show_header(title: &str) {
    println!("\n╔═══════════════════════════════════════════════════╗");
    println!("║ {: ^50}║", title);
    println!("╚═══════════════════════════════════════════════════╝\n");
}

/// Display a list item
pub fn show_list_item(index: usize, item: &str) {
    println!("  {}. {}", index, item);
}

/// Show a table separator
pub fn show_separator(n: u16) {
    println!("  {}", "─".repeat(n as usize));
}

/// Trait for types that can be presented as enum choices in a CLI menu.
/// Implementors supply a human-readable label and a static slice of all variants.
pub trait CliSelectable: Sized + Copy + std::fmt::Display {
    fn label() -> &'static str;
    fn all_variants() -> &'static [Self];
}

impl CliSelectable for ThrowHand {
    fn label() -> &'static str {
        "Throw hand"
    }

    fn all_variants() -> &'static [Self] {
        ThrowHand::all()
    }
}

impl CliSelectable for BatSide {
    fn label() -> &'static str {
        "Bat side"
    }

    fn all_variants() -> &'static [Self] {
        BatSide::all()
    }
}

pub fn choose_enum<T>(current: Option<T>) -> Option<T>
where
    T: CliSelectable + 'static,
{
    let values = T::all_variants();

    let current_display = current
        .map(|c| c.to_string())
        .unwrap_or_else(|| "-".to_string());

    let options_str = values
        .iter()
        .enumerate()
        .map(|(idx, v)| format!("{}:{}", idx + 1, v))
        .collect::<Vec<_>>()
        .join(" ");

    let prompt = format!(
        "{} [{}] ({} 0:keep): ",
        T::label(),
        current_display,
        options_str
    );

    let choice = read_i32(&prompt)?;

    if choice == 0 {
        return None;
    }

    let idx = (choice - 1) as usize;
    values.get(idx).copied()
}

pub fn choose_enum_optional<T>() -> Option<T>
where
    T: CliSelectable + 'static,
{
    let options = T::all_variants();

    let options_str = options
        .iter()
        .enumerate()
        .map(|(idx, v)| format!("{}:{}", idx + 1, v))
        .collect::<Vec<_>>()
        .join(" ");

    let prompt = format!("{} ({} 0:none): ", T::label(), options_str);

    let choice = read_i32(&prompt)?;

    if choice == 0 {
        return None;
    }

    let idx = (choice - 1) as usize;
    options.get(idx).copied()
}

use std::path::PathBuf;

/// Prompts the user for an export directory.
/// Returns None if the user leaves the input blank.
pub fn prompt_export_directory() -> Option<PathBuf> {
    use std::io::{self, Write};

    println!("\n  Output directory (leave empty to cancel):");
    print!("  Path: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        println!("\n❌ Failed to read input.");
        wait_for_enter();
        return None;
    }

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let path = PathBuf::from(trimmed);

    if !path.exists() {
        println!("\n❌ Directory does not exist.");
        wait_for_enter();
        return None;
    }

    if !path.is_dir() {
        println!("\n❌ The specified path is not a directory.");
        wait_for_enter();
        return None;
    }

    Some(path)
}
