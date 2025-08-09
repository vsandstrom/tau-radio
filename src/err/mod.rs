use inline_colorization::*;

#[cfg(target_os="macos")]
pub const DEFAULT_NOT_FOUND: &str = 
      "\n{color_red}Error:{color_reset} {DEFAULT_INPUT} driver not found.\n\
      \nInstall with:\n {color_bright_cyan}$ brew install --cask blackhole-2ch{color_reset}\n\
      or:\n{color_bright_cyan} $ port install BlackHole{color_reset}\n";

pub const AUDIO_INTERFACE_NOT_FOUND: &str = 
      "\nThe audio_interface you have chosen does not match anything on your system. \n\
      Check spelling in config.toml, and make sure it is installed and connected correctly.";
