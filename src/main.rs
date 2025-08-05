mod config;
mod err;

use clap::Parser;
use chrono::Utc;
use std::process::{exit, Command};
use config::load_or_create_config;

#[cfg(target_os = "macos")]
const AUDIO_DRIVER: &str = "avfoundation";
#[cfg(target_os = "macos")]
const DEFAULT_INPUT: &str = "BlackHole 2ch";
#[cfg(target_os = "linux")]
const AUDIO_DRIVER: &str = "alsa";
#[cfg(target_os = "linux")]
const DEFAULT_INPUT: &str = "0";


#[derive(Parser)]
#[command(name = "Tau")]
#[command(version = "0.0.1")]
#[command(about = "Streams to an IceCast server using FFmpeg")]
struct Args {
  /// IceCast server username
  #[arg(short, long, default_value = "mark_fisher")]
  username: String,

  /// IceCast server password
  #[arg(short, long, default_value = "is there no alternative")]
  password: String,

  /// IceCast server URL
  #[arg(short, long, default_value = "127.0.0.1")]
  url: String,

  /// IceCast server port
  #[arg(short, long, default_value = "8000")]
  port: u16,


  /// Optional custom filename of local copy
  #[arg(short, long)]
  file: Option<String>
}

fn main() {
  // parse_drivers();
  // let args = Args::parse();
  let config = load_or_create_config();

  // formats the current datetime to a string, used in session file naming
  // if no filename is given in cli arguments. default = `tau_[datetime].ogg`
  let now = Utc::now().format("%d-%m-%Y_%H:%M:%S").to_string();
  let filename = config.file.map(|f| format!("{f}.ogg")).unwrap_or_else(|| format!("tau_{}.ogg", now));

  // formats the URL to the receiving IceCast server
  let address = format!(
    "icecast://{}:{}@{}:{}/tau.ogg",
    config.username, config.password, config.url, config.port
  );

  // Build the arguments for FFmpeg process
  let ffmpeg_args = [
    "-f", AUDIO_DRIVER,                         // select OS specific audio backend
    "-i", &format!(":{}", get_input_index()),   // select index of audio driver
    "-map", "0:a",                              // extract the audio from the first input stream
    "-c:a", "libopus",                          // encode the audio through 'libopus
    "-b:a", "128k",                             // set bitrate of audiostream
    "-content_type", "audio/opus",              // set HTTP content type tags
    "-f", "tee",                                // split audio into:
    &format!("{}|{}", address, filename)        // icecast stream and local file
  ];

  println!("{}", ffmpeg_args.clone().join(" "));

  let ffmpeg = Command::new("ffmpeg")
    .args(ffmpeg_args)
    .status()
    .expect("FFmpeg failed on startup.");

  if !ffmpeg.success() {
     eprintln!("FFmpeg exited with status: {}", ffmpeg);
     exit(1) 
  }
}

  
#[cfg(target_os = "macos")]
fn get_blackhole_index() -> Option<String> {
  // Run ffmpeg to find the driver index on your system
  let output = Command::new("ffmpeg")
    .args(["-list_devices", "true", "-f", "avfoundation", "-i", "dummy"])
    .output()
    .ok()?;

  // Loop over results to see if `BlackHole 2ch` is installed on your system
  // and parse the index of driver available to FFmpeg.
  let stderr = std::str::from_utf8(&output.stderr).ok()?;
  for line in stderr.lines() {
    if line.contains(DEFAULT_INPUT) {
      if let Some(index) = line
        .split(['[', ']'])
        .filter(|s| !s.is_empty())
        .nth(2) {
        return Some(index.to_string());
      }
    }
  }
  None
}


#[cfg(target_os = "macos")]
/// Find out if `Blackhole 2ch` is available on macOS system,
fn get_input_index() -> String {
    use self::err::BLACKHOLE_NOT_FOUND;

  get_blackhole_index().unwrap_or_else(|| {
    eprintln!("{}", BLACKHOLE_NOT_FOUND);
    exit(1);
  })
}

#[cfg(target_os = "linux")]
/// Choose default ALSA input source
fn get_input_index() -> String {
  DEFAULT_INPUT.to_string()
}

#[cfg(target_os = "macos")]
#[allow(unused)]
fn parse_drivers() {
  // Run ffmpeg to find the driver index on your system
  let output = Command::new("ffmpeg")
    .args(["-list_devices", "true", "-f", "avfoundation", "-i", "dummy"])
    .output()
    .ok().unwrap();

  // Loop over results to see if `BlackHole 2ch` is installed on your system
  // and parse the index of driver available to FFmpeg.
  let stderr = std::str::from_utf8(&output.stderr).ok().unwrap();
  for line in stderr.lines() {
    if line.contains(DEFAULT_INPUT) {
      if let Some(l) = line .split(['[', ']']).nth(3) { 
        print!("\t{}", l)
      };
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(target_os = "macos")]
  #[test]
  fn get_mac_output_driver() {
        let output = Command::new("ffmpeg")
        .args(["-list_devices", "true", "-f", "avfoundation", "-i", "dummy"])
        .output()
        .ok().unwrap();

    let stderr = std::str::from_utf8(&output.stderr).ok().unwrap();
    let mut found = false;
    for line in stderr.lines() {
      if line.contains(DEFAULT_INPUT) {
        found = true;
      }
    }
    assert!(found)
  }
  
}
