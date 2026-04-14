#![deny(unused_imports, unused_extern_crates)]
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
use crate::util::create_recordings_dir;

use clap::Parser;
use cpal::{
  SampleRate,
  traits::{DeviceTrait, StreamTrait},
  StreamConfig
};

use inline_colorization::*;
use ringbuf::{
   HeapRb, 
   traits::{Producer, Split}
};

use std::{
  path::PathBuf,
  sync::{Arc, atomic::{AtomicBool, Ordering}},
  thread
};

use util::consts::{DEFAULT_CH, DEFAULT_SR, DEFAULT_INPUT};
use config::Credentials;


fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let output = &args.output.clone();
  let config = Config::load_or_create(args.reset_config).map(|c| c.merge_cli_args(&args))?;
  let shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
  let filename = crate::util::format_filename(config.file.clone());
  let home = std::env::var("HOME")?;
  let record_path = match output {
    Some(p) => PathBuf::from(p),
    None => PathBuf::from(home).join("tau").join("recordings"),
  };

  if !record_path.exists() && let Err(e) = create_recordings_dir(&record_path) {
    return Err(
      anyhow::anyhow!(
        "{}Could not create directory for saving recorded sessions: {}\n\t{}{}{}\n\n{e}",
        color_yellow,
        color_reset,
        color_red,
        record_path.display(),
        color_reset
      )
    );
  }

  let path = record_path.join(filename.clone().to_string());
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

  let (mut tx, rx) = HeapRb::<f32>::new(DEFAULT_SR as usize * 4)
    .split();

  let credentials: Credentials = Credentials::new(
    config.username.clone(),
    config.password.clone(),
  );

  {
    let url = config.url.clone();
    let shutdown = shutdown.clone();
    if args.no_recording {
      thread::spawn({
        move ||
          ws::thread( 
            rx,
            (&url, config.upstream_port),
            config.tls,
            filename,
            credentials,
            shutdown
          )
      });
    } else {
      thread::spawn({
        move || 
          ws::rec_thread(
            rx,
            (&url, config.upstream_port),
            config.tls,
            &record_path,
            filename,
            credentials,
            shutdown
          )
      });
    }
  }

  let requested_config = StreamConfig {
    channels: DEFAULT_CH as u16,
    sample_rate: SampleRate(DEFAULT_SR as u32),
    buffer_size: cpal::BufferSize::Default,
  };
 
  let host = cpal::default_host();
  let device = crate::audio::find_audio_device(&host, &config.audio_interface)?;
  let dev_cfg = device.default_input_config()?;
  dbg!(dev_cfg.config());
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
    &config.url,
    &config.upstream_port,
  );

  loop {
    if shutdown.load(Ordering::Acquire) { return Ok(()) }
    thread::sleep(std::time::Duration::from_millis(100));
  }
}
