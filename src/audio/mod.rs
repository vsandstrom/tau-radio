use crate::config::Config;
use crate::{AUDIO_INTERFACE_NOT_FOUND, DEFAULT_INPUT, err::default_not_found};

use cpal::{
    Device, Host,
    traits::{DeviceTrait, HostTrait},
};

pub fn find_audio_device(host: &Host, config: &Config) -> anyhow::Result<Device> {
  let devices = host.input_devices().map_err(|err| {
    anyhow::anyhow!(
      "Could not list input devices: {err}\n\
       Make sure your audio hardware is connected and accessible"
    )
  })?;
  if let Some(dev) = devices
    .filter_map(|d| d.name().ok().map(|n| (d, n)))
    .find(|(_, name)| name == &config.audio_interface)
    .map(|(d, _)| d)
  {
    return Ok(dev);
  }
  if config.audio_interface == DEFAULT_INPUT {
    Err(anyhow::anyhow!("{}", default_not_found()))
  } else {
    Err(anyhow::anyhow!("{}", AUDIO_INTERFACE_NOT_FOUND))
  }
}
