mod args;
mod config;
mod err;
mod icecast;
mod util;

use crate::args::Args;
use crate::config::Config;
use crate::err::{AUDIO_INTERFACE_NOT_FOUND, DEFAULT_NOT_FOUND};
use crate::icecast::create_icecast_connection;
use crate::util::format_filename;

use clap::Parser;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, InputCallbackInfo, SupportedStreamConfig};
#[allow(unused)]
use inline_colorization::*;
use opusenc::{Comments, Encoder, RecommendedTag};
use ringbuf::traits::{Consumer, Producer, Split};
use std::process::exit;


#[cfg(target_os = "macos")]
const DEFAULT_INPUT: &str = "BlackHole 2ch";
#[cfg(target_os = "linux")]
const DEFAULT_INPUT: &str = "jack";

fn main() {
  let args = Args::parse();
  let config = Config::load_or_create(args.reset_config).merge_cli_args(args);
  // merge_cli_args(&mut config, args);
  let filename = format_filename(config.file.clone());
  let host = cpal::default_host();
  let device = find_audio_device(&host, &config);
  let audio_config  = device.default_input_config()
    .unwrap_or_else(|err| { eprintln!("Failed to poll default supported config from audio device: {err}"); exit(1) });
  let (sr, ch) = get_device_settings(&audio_config);
  let (mut tx, mut rx) = ringbuf::HeapRb::<f32>::new(sr as usize * 4).split();
  let icecast = create_icecast_connection(config.clone());
  let inner_filename = filename.clone();

  let _stream_thread = if config.no_recording {
    std::thread::spawn(move || {
      let mut encoder = Encoder::create_pull(
        Comments::create().add(RecommendedTag::Title, inner_filename.to_string()).unwrap(),
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

  println!("\n{style_bold}{color_bright_yellow}Recording from: \t{style_reset}{color_bright_cyan}{}{color_reset}", device.name().unwrap_or("Unknown device".into()));
  println!("{style_bold}{color_bright_yellow}Streaming live to: \t{color_bright_cyan}http://{}:{}/tau.ogg{color_reset}", config.url, config.port);
  if !config.no_recording { 
    println!("{style_bold}{color_bright_yellow}Saving local copy to: \t{style_reset}{color_bright_cyan}{}{color_reset}", filename); 
  } else { 
    println!("{color_red}{style_bold}Local recording is disabled.{style_reset}{color_reset}"); 
  }
  println!("Press Ctrl+C to stop.");

  loop { std::thread::sleep(std::time::Duration::from_secs(1)); }
}

fn get_device_settings(config: &SupportedStreamConfig) -> (u32, u16) {
  let sr = config.config().sample_rate.0;
  if sr != 48_000 {
      eprintln!("The selected audio device is set to {sr} Hz.");
      eprintln!("Opus streaming requires exactly 48,000 Hz.");
      eprintln!("Please adjust your system audio settings and try again.");
      exit(1);
  }

  let ch = config.config().channels;
  (sr, ch)
}

fn find_audio_device(host: &Host, config: &Config) -> Device {
  host.input_devices()
      .map_err(|err| {
          eprintln!("Could not list input devices: {err}");
          eprintln!("Make sure your audio hardware is connected and accessible.");
          exit(1);
      })
      .ok()
      .and_then(|mut devs| 
        devs.find(|dev| 
          dev.name().unwrap_or_default() == config.audio_interface
        )
      )
      .unwrap_or_else(|| {
        if config.audio_interface == DEFAULT_INPUT { eprintln!("{}", DEFAULT_NOT_FOUND); }
        else { eprintln!("{}", AUDIO_INTERFACE_NOT_FOUND);}
        exit(1);
      })
}
