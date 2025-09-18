use clap::Parser;

use crate::StreamType;
use crate::config::TauConfigError;
use is_ip::is_ip;

#[derive(Parser)]
#[command(name = "Tau")]
#[command(version = "0.0.1")]
#[command( about = "Hijacks chosen audio device, encodes audio into Ogg Opus and streams to IceCast server")]
pub(crate) struct Args {
    /// IceCast server username
    #[arg(long)]
    pub username: Option<String>,

    /// IceCast server password
    #[arg(long)]
    pub password: Option<String>,

    /// IceCast server URL
    #[arg(short, long, value_parser=|s: &str| validate_ip(s.to_string()))]
    pub url: Option<String>,

    /// IceCast server port
    #[arg(short, long, value_parser=|p: &str| {
      validate_port(parse_port(p).unwrap())
    })]
    pub port: Option<u16>,

    #[arg(short, long)]
    pub mount: Option<String>,

    /// Optional custom filename of local copy
    #[arg(short, long)]
    pub file: Option<String>,

    /// Disables the local recording of stream
    #[arg(long)]
    pub no_recording: bool,

    #[arg(short, long)]
    pub output: Option<String>,

    #[arg(long)]
    pub reset_config: bool,
    // #[arg(long, value_parser=validate_stream_type)]
    // pub stream_mode: crate::StreamType
}

pub fn validate_ip(url: String) -> Result<String, TauConfigError> {
  if !is_ip(&url) {
    return Err(TauConfigError::InvalidIp(url));
  }
  Ok(url)
}

fn parse_port(p: &str) -> Result<u16, TauConfigError> {
  p.parse::<u16>()
    .map_err(|e| TauConfigError::Input(format!("Unable to parse as number: {e}")))
}

pub fn validate_port(port: u16) -> Result<u16, TauConfigError> {
  if !(1..=0xFFFF).contains(&port) {
    return Err(TauConfigError::InvalidPort(port));
  }
  Ok(port)
}

pub fn validate_stream_type(t: &str) -> Result<StreamType, TauConfigError> {
  match t {
    "icecast" => Ok(StreamType::IceCast),
    "websocket" => Ok(StreamType::WebSocket),
    _ => Err(TauConfigError::Input("Invalid streaming type".to_string())),
  }
}
