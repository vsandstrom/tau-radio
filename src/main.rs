#![deny(unused_extern_crates)]
mod args;
mod audio;
mod config;
mod err;
mod threads;
mod ui;
mod util;

use crate::args::Args;
use crate::config::Config;
use crate::err::AUDIO_INTERFACE_NOT_FOUND;
use crate::threads::ws;

use clap::Parser;
use cpal::{
  SampleRate,
  traits::{DeviceTrait, StreamTrait},
  StreamConfig
};
#[allow(unused)]
use inline_colorization::*;
use ringbuf::{
  HeapRb,
  traits::{Producer, Split},
};
use std::{
  net::{Ipv4Addr, SocketAddr},
  path::PathBuf,
  str::FromStr,
  sync::{atomic::{AtomicBool, Ordering}, Arc},
  thread::spawn
};

#[cfg(target_os = "macos")]
const DEFAULT_INPUT: &str = "BlackHole 2ch";
#[cfg(target_os = "linux")]
const DEFAULT_INPUT: &str = "pipewire";

const DEFAULT_SR: i32 = 48000;
// TODO: Handle multichannel stream based on user config
const DEFAULT_CH: usize = 2;


struct Credentials {
  username: String,
  password: String,
  broadcast_port: u16,
}

fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let output = &args.output.clone();
  let config = Config::load_or_create(args.reset_config).map(|c| c.merge_cli_args(&args))?;
  let filename = crate::util::format_filename(config.file.clone());
  let home = std::env::var("HOME")?;
  let out_dir = match output {
    Some(p) => PathBuf::from(p),
    None => PathBuf::from(home).join("tau").join("recordings"),
  };

  let shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

  let path = out_dir.join(filename.clone().to_string());
  if path.exists() {
    return Err(anyhow::anyhow!(
      "{}\n\tUnable to overwrite already existing file:{}\n\t{}{}{}",
      color_yellow,
      color_reset,
      color_red,
      path.display(),
      color_reset
    ));
  }

  let host = cpal::default_host();
  let device = crate::audio::find_audio_device(&host, &config.audio_interface)?;
  let (mut tx, rx) = HeapRb::<f32>::new(DEFAULT_SR as usize * 4).split();
  let remote_ip = Ipv4Addr::from_str(&config.ip)?;
  let remote_addr = SocketAddr::new(std::net::IpAddr::V4(remote_ip), config.port);

  let creds: Credentials = Credentials { 
    username: config.username.clone(), 
    password: config.password.clone(),
    broadcast_port: config.broadcast_port
  };

  let filename = filename.clone();
  let shutdown_clone = shutdown.clone();
  if args.no_recording {
    spawn(move || ws::thread( rx, remote_addr, filename, creds, shutdown_clone));
  } else {
    spawn(move || ws::rec_thread( rx, remote_addr, &out_dir, filename, creds, shutdown_clone));
  }

  let requested_config = StreamConfig {
    channels: DEFAULT_CH as u16,
    sample_rate: SampleRate(DEFAULT_SR as u32),
    buffer_size: cpal::BufferSize::Default,
  };

  let stream = device
    .build_input_stream(
      &requested_config,
      move |buf, _info| {
        tx.push_slice(buf);
      },
      |e| {
        eprintln!("{e}");
        std::process::exit(1)
      },
      None,
    )
    .map_err(crate::err::handle_input_build_error)?;

  stream.play()?;

  // Prints pretty message
  crate::ui::print_started_session_msg(
    config.audio_interface,
    &path,
    args.no_recording,
    &config.ip,
    &config.port,
  );

  

  loop {
    if shutdown.load(Ordering::SeqCst) { return Ok(()) }
    std::thread::sleep(std::time::Duration::from_millis(100));
  }

}
