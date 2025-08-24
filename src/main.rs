mod args;
mod config;
mod err;
mod audio;
mod util;
mod ui;

use crate::args::Args;
use crate::config::Config;
use crate::err::{AUDIO_INTERFACE_NOT_FOUND, DEFAULT_NOT_FOUND, handle_input_build_error};
use crate::audio::{create_icecast_connection, find_audio_device};
use crate::util::format_filename;

use clap::Parser;
use cpal::{
  InputCallbackInfo, SampleRate,
  traits::{ DeviceTrait, StreamTrait }
};

#[allow(unused)]
use inline_colorization::*;
use opusenc::{Comments, Encoder, RecommendedTag};
use ringbuf::traits::{Consumer, Producer, Split};
use std::process::exit;
use cpal::StreamConfig;


#[cfg(target_os = "macos")]
const DEFAULT_INPUT: &str = "BlackHole 2ch";
#[cfg(target_os = "linux")]
const DEFAULT_INPUT: &str = "pipewire";

const DEFAULT_SR: i32 = 48000;
// TODO: Handle multichannel stream based on user config
const DEFAULT_CH: usize = 2;

fn main() -> anyhow::Result<()> {
  let args = Args::parse();
  let config = Config::load_or_create(args.reset_config)
    .map(|c| c.merge_cli_args(args))?;
  let filename = format_filename(config.file.clone());
  let host = cpal::default_host();
  let device = find_audio_device(&host, &config)?;
  let (mut tx, mut rx) = ringbuf::HeapRb::<f32>::new(DEFAULT_SR as usize * 4).split();
  let icecast = create_icecast_connection(config.clone())?;
  let inner_filename = filename.clone();

  let _stream_thread = if config.no_recording {
    std::thread::spawn(move || {
      let mut encoder = Encoder::create_pull(
        Comments::create()
          .add(RecommendedTag::Title, inner_filename.to_string())
          .unwrap(),
        DEFAULT_SR,
        DEFAULT_CH,
        opusenc::MappingFamily::MonoStereo).unwrap_or_else(|err| {
          eprintln!("Could not create new realtime .ogg encoder: {err}");
          exit(1)
      });
    
      let framesize = 960 * DEFAULT_CH;
      let mut opus_frame_buffer = Vec::with_capacity(framesize);
      loop {
        if let Some(sample) = rx.try_pop() {
          opus_frame_buffer.push(sample);
        } else {
          // If no samples are available, let CPU breath
          std::thread::sleep(std::time::Duration::from_millis(2));
        }
        if opus_frame_buffer.len() == framesize {
          encoder
            .write_float(&opus_frame_buffer)
            .expect("block not a multiple of input channels");
          // flush forces encoder to return a page, even if not ready. 
          // true is used when realtime streaming is more important than stability. 
          if let Some(page) = encoder.get_page(true) {
            icecast.send(page).unwrap();
            icecast.sync();
          }
          opus_frame_buffer.clear();
        }
      }
    })
  } else {
    std::thread::spawn(move || {
      let mut local_encoder = Encoder::create_file(
        inner_filename.clone().as_str(),
        Comments::create()
          .add(RecommendedTag::Title, inner_filename.clone().to_string())
          .unwrap(),
        DEFAULT_SR,
        DEFAULT_CH,
        opusenc::MappingFamily::MonoStereo).unwrap_or_else(|err| {
          eprintln!("Could not create new local .ogg file: {err}");
          exit(1)
      });

      let mut stream_encoder = Encoder::create_pull(
        Comments::create()
          .add(RecommendedTag::Title, inner_filename.clone().to_string())
          .unwrap(),
        DEFAULT_SR,
        DEFAULT_CH,
        opusenc::MappingFamily::MonoStereo)
        .unwrap_or_else(|err| {
          eprintln!("Could not create new realtime .ogg encoder: {err}");
          exit(1)
      });
    
      let framesize = 960 * DEFAULT_CH;
      let mut opus_frame_buffer = Vec::with_capacity(framesize);
      loop {
        if let Some(sample) = rx.try_pop() {
          opus_frame_buffer.push(sample);
        } else {
          std::thread::sleep(std::time::Duration::from_millis(2));
        }
        if opus_frame_buffer.len() == framesize {
          local_encoder
            .write_float(&opus_frame_buffer)
            .expect("block not a multiple of input channels");
          stream_encoder
            .write_float(&opus_frame_buffer)
            .expect("block not a multiple of input channels");
          // flush forces encoder to return a page, even if not ready. 
          // true is used when realtime streaming is more important than stability. 
          if let Some(page) = stream_encoder.get_page(true) {
            icecast.send(page).unwrap();
            icecast.sync();
          }
          opus_frame_buffer.clear();
        }
      }
    })
  };

  let requested_config = StreamConfig{
    channels: DEFAULT_CH as u16,
    sample_rate: SampleRate(DEFAULT_SR as u32),
    buffer_size: cpal::BufferSize::Default
  };

  let input_cb= move |buf: &[f32], _info: &InputCallbackInfo| { tx.push_slice(buf); };
  let err_cb = |err| {eprintln!("{err}")};
  let stream  = device
    .build_input_stream(
      &requested_config, 
      input_cb,
      err_cb,
      None
    )
    .map_err(handle_input_build_error)?;
  let _ = stream.play();

  crate::ui::print_started_session_msg(config.audio_interface, &config.url, &config.port, &filename, config.no_recording);

  loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
}


