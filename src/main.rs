use bs_scoring::cli::screens::main_menu;
use bs_scoring::utils::boot::{boot_screen_footer, boot_screen_header};
use bs_scoring::{setup_db, utils};

fn main() {
    utils::term::clear_screen();
    println!();

    boot_screen_header();

    let (mut db, db_path, status) = match setup_db() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("\n❌ Boot failed:\n{e:#}");
            utils::term::wait_for_enter();
            std::process::exit(1);
        }
    };

    boot_screen_footer(&db_path, &status);

    // qui ha senso una pausa “umana”
    utils::term::wait_for_enter();

    main_menu::run_main_menu(&mut db);
}
