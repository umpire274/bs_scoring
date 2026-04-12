/// Returns a compact timestamp suitable for export filenames.
pub fn export_timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}
