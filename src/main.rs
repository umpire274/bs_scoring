use bs_scoring::cli::commands::main_menu;
use bs_scoring::setup_db;

fn main() {
    let db = setup_db();
    std::thread::sleep(std::time::Duration::from_secs(3));
    println!();

    main_menu::run_main_menu(&db);
}
