use crate::Database;
use crate::utils::term;

pub fn handle_statistics(_db: &Database) {
    term::show_header("STATISTICS");
    println!("🚧 Statistics module under development...\n");
    println!("  Here you will be able to view:");
    println!("  - Player statistics");
    println!("  - Batting average, ERA, OPS");
    println!("  - League standings");
    println!("  - Game history\n");
    term::wait_for_enter();
}
