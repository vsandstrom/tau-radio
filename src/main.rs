use clap::Parser;
use chrono::Utc;
use std::process::{exit, Command};
use inline_colorization::*;

#[cfg(target_os = "macos")]
const AUDIO_DRIVER: &str = "avfoundation";

#[cfg(target_os = "linux")]
const AUDIO_DRIVER: &str = "alsa";

const MAC_AUDIO_DRIVER: &str = "BlackHole 2ch";

#[derive(Parser)]
#[command(name = "Tau")]
#[command(version = "0.0.1")]
#[command(about = "Streams to an IceCast server using FFmpeg")]
struct Args {
  /// IceCast server username
  #[arg( long, default_value = "mark_fisher")]
  username: String,

  /// IceCast server password
  #[arg(long, default_value = "is there no alternative")]
  password: String,

  /// IceCast server URL
  #[arg(long, default_value = "127.0.0.1")]
  url: String,

  /// IceCast server port
  #[arg(long, default_value = "8000")]
  port: u16,

  /// Optional custom filename of local copy
  #[arg(long)]
  file: Option<String>
}

fn main() {
  // parse_drivers();
  let args = Args::parse();

  // formats the current datetime to a string, used in session file naming
  // if no filename is given in cli arguments. default = `tau_[datetime].ogg`
  let now = Utc::now().format("%d-%m-%Y_%H:%M:%S").to_string();
  let filename = args.file.map(|f| format!("{f}.ogg")).unwrap_or_else(|| format!("tau_{}.ogg", now));

  // formats the URL to the receiving IceCast server
  let address = format!(
    "icecast://{}:{}@{}:{}/tau.ogg",
    args.username, args.password, args.url, args.port
  );

  // Build the arguments for FFmpeg process
  let ffmpeg_args = [
    "-f", AUDIO_DRIVER,                         // select OS specific audio backend
    "-i", &format!(":{}", get_input_index()),   // select index of audio driver
    "-map", "0:a",                              // extract the audio from the first input stream
    "-c:a", "libopus",                          // encode the audio through 'libopus
    "-b:a", "128k",                             // set bitrate of audiostream
    // "-content_type", "audio/opus",              // set HTTP content type tags
    "-f", "tee",                                // split audio into:
    &format!("'[f=ogg]{}|[f=ogg]{}'", address, filename)        // icecast stream and local file
  ];

  // println!("{}", ffmpeg_args.clone().join(" "));

  let ffmpeg = Command::new("ffmpeg")
    .args(ffmpeg_args)
    .status()
    .expect("FFmpeg failed on startup.");

  if !ffmpeg.success() {
     eprintln!("FFmpeg exited with status: {}", ffmpeg);
     exit(1) 
  }

  //// spawn asciinema
  // let mut ffmpeg = Command::new("ffmpeg")
  // .args(ffmpeg_args)
  // .stdout(Stdio::null())
  // .stderr(Stdio::null())
  // .spawn()
  // .expect("FFmpeg failed on startup.");

  // let asciinema = Command::new("asciinema")
  //   .args(["rec"])
  //   .status()
  //   .expect("Asciinema failed on startup.");
  //
  // if asciinema.success() {
  //   let _ = ffmpeg.kill();
  //   }
}

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
    if line.contains(MAC_AUDIO_DRIVER) {
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
  get_blackhole_index().unwrap_or_else(|| {
    eprintln!("\n{color_red}Error:{color_reset} {MAC_AUDIO_DRIVER} driver not found.\n\
      \nInstall with:\n {color_bright_cyan}$ brew install --cask blackhole-2ch{color_reset}\n\
      or:\n{color_bright_cyan} $ port install BlackHole{color_reset}\n");
    exit(1);
  })
}

#[cfg(target_os = "linux")]
/// Choose default ALSA input source
fn get_input_index() -> String {
  "0".to_string()
}

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
    if line.contains(MAC_AUDIO_DRIVER) {
      line .split(['[', ']']).nth(3).map(|l| print!("\t{}", l));
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
      if line.contains(MAC_AUDIO_DRIVER) {
        found = true;
      }
    }
    assert!(found)
  }
  
}
