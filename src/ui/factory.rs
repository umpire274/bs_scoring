use crate::ui::Ui;
use crate::ui::cli_impl::CliUi;
use crate::ui::tui::TuiUi;
use crate::utils::term;

pub fn create_ui() -> Box<dyn Ui> {
    match TuiUi::new() {
        Ok(tui) => Box::new(tui),
        Err(e) => {
            term::show_error(&format!(
                "Failed to initialize TUI (falling back to CLI): {e}"
            ));
            Box::new(CliUi::new())
        }
    }
}
