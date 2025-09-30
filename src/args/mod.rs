use clap::Parser;

// use crate::StreamType;
use crate::config::TauConfigError;
use is_ip::is_ip;

#[derive(Parser)]
#[command(name = "tau-radio")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command( about = "Hijacks chosen audio device, encodes audio into Ogg Opus and streams to webradio server [tau-tower]")]
pub(crate) struct Args {
    /// Webradio server username
    #[arg(long)]
    pub username: Option<String>,

    /// Webradio server password
    #[arg(long)]
    pub password: Option<String>,

    /// Tau-tower server ip
    #[arg(short, long, value_parser=|s: &str| validate_ip(s.to_string()))]
    pub ip: Option<String>,

    /// Tau-tower server port
    #[arg(short, long, value_parser=|p: &str| {
      validate_port(parse_port(p).unwrap())
    })]
    pub port: Option<u16>,

    /// Optional custom filename of local copy
    #[arg(short, long)]
    pub file: Option<String>,

    /// Disables the local recording of stream
    #[arg(long)]
    pub no_recording: bool,

    /// Output directory [default: $HOME/tau/recordings/]
    #[arg(short, long)]
    pub output: Option<String>,

    /// Resets config.toml 
    #[arg(long)]
    pub reset_config: bool,
}

pub fn validate_ip(ip: String) -> Result<String, TauConfigError> {
  if !is_ip(&ip) {
    return Err(TauConfigError::InvalidIp(ip));
  }
  Ok(ip)
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

// pub fn validate_stream_type(t: &str) -> Result<StreamType, TauConfigError> {
//   match t {
//     "icecast" => Ok(StreamType::IceCast),
//     "websocket" => Ok(StreamType::WebSocket),
//     _ => Err(TauConfigError::Input("Invalid streaming type".to_string())),
//   }
// }
