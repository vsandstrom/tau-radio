mod config;
mod args;
mod err;
mod util;
mod icecast;

use crate::config::load_or_create_config;
use crate::config::merge_cli_args;
use crate::util::format_filename;
use crate::args::Args;

use chrono::Local;
use clap::Parser;
use std::process::exit;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::InputCallbackInfo;
use opusenc::{Comments, Encoder, RecommendedTag};
use ringbuf::traits::{Consumer, Producer, Split};
use std::sync::Arc;
use crate::err::{AUDIO_INTERFACE_NOT_FOUND, DEFAULT_NOT_FOUND};
use self::icecast::create_icecast_connection;

#[cfg(target_os = "macos")]
const DEFAULT_INPUT: &str = "BlackHole 2ch";

fn main() {
  let args = Args::parse();
  let mut config = load_or_create_config(args.reset_config);
  merge_cli_args(&mut config, args);
  let filename = format_filename(config.file.clone());
  let host = cpal::default_host();

  let device = host.input_devices()
      .map_err(|err| {
          eprintln!("Could not list input devices: {err}");
          eprintln!("Make sure your audio hardware is connected and accessible.");
          exit(1);
      })
      .ok()
      .and_then(|mut devs| devs.find(|dev| dev.name().unwrap_or_default() == config.audio_interface))
      .unwrap_or_else(|| {
        if config.audio_interface == DEFAULT_INPUT { eprintln!("{}", DEFAULT_NOT_FOUND); }
        else { eprintln!("{}", AUDIO_INTERFACE_NOT_FOUND);}
        exit(1);
      });

  let audio_config  = device.default_input_config()
    .unwrap_or_else(|err| { eprintln!("Failed to poll default supported config from audio device: {err}"); exit(1) });

  let sr = audio_config.config().sample_rate.0;
  if sr != 48_000 {
      eprintln!("The selected audio device is set to {sr} Hz.");
      eprintln!("Opus streaming requires exactly 48,000 Hz.");
      eprintln!("Please adjust your system audio settings and try again.");
      exit(1);
  }

  let ch = audio_config.config().channels;
  let (mut tx, mut rx) = ringbuf::HeapRb::<f32>::new(sr as usize * 4).split();
  let icecast = create_icecast_connection(config.clone());
  let inner_filename = filename.clone();

  let _stream_thread = if config.no_recording {
    std::thread::spawn(move || {
      let mut encoder = Encoder::create_pull(
        Comments::create().add(RecommendedTag::Title, inner_filename.clone().to_string()).unwrap(),
        sr as i32,
        ch as usize,
        opusenc::MappingFamily::MonoStereo).unwrap_or_else(|err| {
          eprintln!("Could not create new realtime .ogg encoder: {err}");
          exit(1)
      });
    
      let framesize = 960 * ch as usize;
      let mut opus_frame_buffer = Vec::with_capacity(framesize);
      loop {
        if let Some(sample) = rx.try_pop() {
          opus_frame_buffer.push(sample);
        } else {
          // If no samples are available, let CPU breath
          std::thread::sleep(std::time::Duration::from_millis(2));
        }
        if opus_frame_buffer.len() == framesize {
          encoder.write_float(&opus_frame_buffer).expect("block not a multiple of input channels");
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
        Comments::create().add(RecommendedTag::Title, inner_filename.clone().to_string()).unwrap(),
        sr as i32,
        ch as usize,
        opusenc::MappingFamily::MonoStereo).unwrap_or_else(|err| {
          eprintln!("Could not create new local .ogg file: {err}");
          exit(1)
      });

      let mut stream_encoder = Encoder::create_pull(
        Comments::create().add(RecommendedTag::Title, inner_filename.clone().to_string()).unwrap(),
        sr as i32,
        ch as usize,
        opusenc::MappingFamily::MonoStereo).unwrap_or_else(|err| {
          eprintln!("Could not create new realtime .ogg encoder: {err}");
          exit(1)
      });
    
      let framesize = 960 * ch as usize;
      let mut opus_frame_buffer = Vec::with_capacity(framesize);
      loop {
        if let Some(sample) = rx.try_pop() {
          opus_frame_buffer.push(sample);
        } else {
          std::thread::sleep(std::time::Duration::from_millis(2));
        }
        if opus_frame_buffer.len() == framesize {
          local_encoder.write_float(&opus_frame_buffer).expect("block not a multiple of input channels");
          stream_encoder.write_float(&opus_frame_buffer).expect("block not a multiple of input channels");
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

  let input_cb= move |buf: &[f32], _info: &InputCallbackInfo| { tx.push_slice(buf); };
  let err_cb = |err| {eprintln!("{err}")};
  let stream  = device.build_input_stream(&audio_config.config(), input_cb, err_cb, None).expect("could not build audio capture");
  let _ = stream.play();

  println!("Recording from: {}", device.name().unwrap_or("Unknown device".into()));
  println!("Streaming live to: http://{}:{}/tau.ogg", config.url, config.port);
  if !config.no_recording { println!("Saving local copy to: {}", filename); } 
  else { println!("Local recording is disabled."); }
  println!("Press Ctrl+C to stop.");
  loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
}
