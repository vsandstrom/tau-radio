use super::{Arc, Local};
use regex::Regex;

/// Formats the file name of output file, using current local datetime.
/// If no filename is given in cli arguments. default = `tau_[datetime].ogg`
pub fn format_filename(filename: Option<String>) -> Arc<String> {
  let now = Local::now().format("%d-%m-%Y_%H_%M_%S").to_string();
  Arc::new(filename.map(|f| format!("{f}.ogg")).unwrap_or_else(|| format!("tau_{}.ogg", now)))
}


pub fn validate_port(p: &str) -> Result<u16, String> {
    let port = p.parse::<u16>().map_err(|_| "Invalid port number")?;
    if !(1..=0xFFFF).contains(&port) {
        Ok(port)
    } else {
        Err("Port number must be between 1 and 65535".to_owned())
    }
}
