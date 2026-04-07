use inline_colorization::{
  color_bright_yellow,
  color_bright_red,
  color_reset
};
use std::path::PathBuf;

pub fn config_created(path: &PathBuf) {
  println!("\
    \n{color_bright_yellow}A config file has been written to:{color_reset}\n\t\
    {color_bright_red}{}{color_reset}\n", 
    path.display()
  );
}

fn prompt(msg: &str) -> String { format!("{color_bright_yellow}{msg}{color_reset}") }
pub fn filename_prompt() -> String { prompt("Filename (leave empty for 'tau_[timestamp].ogg')") }
pub fn audio_interface_prompt() -> String {prompt("Audio Interface") }
pub fn port_prompt() -> String { prompt("Upstream Port") }
pub fn ip_prompt() -> String { prompt("Broadcast IP or URL") }
pub fn password_prompt() -> String { prompt("Password") }
pub fn username_prompt() -> String { prompt("Username") }
pub fn tls_prompt() -> String { prompt("SSL/TLS enabled") }

pub fn config_not_found(path: &PathBuf) {
  println!(
    "\n{color_bright_yellow}No config found at '{}'. Let's create one: {color_reset}",
    path.display()
  );
}

pub fn warn_about_credentials() {
  println!("{color_bright_red}Credentials must correspond to broadcast server stream config{color_reset}\n");
}
