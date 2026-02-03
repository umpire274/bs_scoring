use std::io;
use std::io::Write;

/// Clear the screen (works on most terminals)
pub(crate) fn clear_screen() {
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

/// Read an i64
pub fn read_i64(prompt: &str) -> Option<i64> {
    let input = read_string(prompt);
    input.parse().ok()
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
pub fn show_separator() {
    println!("  ─────────────────────────────────────────────────");
}
