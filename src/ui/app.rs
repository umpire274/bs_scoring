pub struct App {
    pub log_scroll: u16,
    pub auto_scroll_log: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            log_scroll: 0,
            auto_scroll_log: true,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
