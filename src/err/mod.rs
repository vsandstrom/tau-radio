use cpal::BuildStreamError;
use cpal::BackendSpecificError;
use std::error::Error;
#[allow(unused)]
use inline_colorization::{color_red, color_bright_cyan, color_reset};

#[cfg(target_os="macos")]
pub const DEFAULT_NOT_FOUND: &str = 
      "\n{color_red}Error:{color_reset} {DEFAULT_INPUT} driver not found.\n\
      \nInstall with:\n {color_bright_cyan}$ brew install --cask blackhole-2ch{color_reset}\n\
      or:\n{color_bright_cyan} $ port install BlackHole{color_reset}\n";

#[cfg(target_os="linux")]
pub const DEFAULT_NOT_FOUND: &str = 
      "\n{color_red}Error:{color_reset} {DEFAULT_INPUT} driver not found.\n\
      \nInstall with:\n {color_bright_cyan}$ brew install --cask blackhole-2ch{color_reset}\n\
      or:\n{color_bright_cyan} $ port install BlackHole{color_reset}\n";

pub const AUDIO_INTERFACE_NOT_FOUND: &str = 
      "\nThe audio_interface you have chosen does not match anything on your system. \n\
      Check spelling in config.toml, and make sure it is installed and connected correctly.";


pub fn handle_input_build_error(err: BuildStreamError) -> anyhow::Error {
  match err {
    BuildStreamError::StreamConfigNotSupported => anyhow::anyhow!(
      "StreamConfigNotSupported: \n\
      \tSome requirements for running Tau is not met by your audio source. \n\
      \tCheck samplerate, it should be 48kHz.\
      \n\tPlease adjust your system audio settings and try again."),
    BuildStreamError::InvalidArgument => anyhow::anyhow!(
      "Argument to underlying C-functions were not understood."),
    BuildStreamError::StreamIdOverflow => anyhow::anyhow!(
      "ID of stream caused an integer overflow."),
    BuildStreamError::DeviceNotAvailable => anyhow::anyhow!(
      "Audio Device is no longer available."),
    BuildStreamError::BackendSpecific { err } => anyhow::anyhow!(
      "BackendSpecificError: {}\n\n{:?}",
      err.description, 
      err.source())
  }
}
