use chrono::Local;
use std::sync::Arc;

/// Formats the file name of output file, using current local datetime.
/// If no filename is given in cli arguments. default = `tau_[datetime].ogg`
pub fn format_filename(filename: Option<String>) -> Arc<String> {
    let now = Local::now().format("%d-%m-%Y_%H_%M_%S").to_string();
    Arc::new(
        filename
            .map(|f| format!("{f}.ogg"))
            .unwrap_or_else(|| format!("tau_{}.ogg", now)),
    )
}
