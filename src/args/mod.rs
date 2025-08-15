use clap::Parser;

#[derive(Parser)]
#[command(name = "Tau")]
#[command(version = "0.0.1")]
#[command(about = "Streams to an IceCast server using FFmpeg")]
pub(crate) struct Args {
  /// IceCast server username
  #[arg(long)]
  pub username: Option<String>,

  /// IceCast server password
  #[arg(long)]
  pub password: Option<String>,

  /// IceCast server URL
  #[arg(long, value_parser=validate_ip)]
  pub url: Option<String>,

  /// IceCast server port
  #[arg(long, value_parser=validate_port)]
  pub port: Option<u16>,

  /// Optional custom filename of local copy
  #[arg(long)]
  pub file: Option<String>,

  /// Disables the local recording of stream
  #[arg(long)]
  pub no_recording: Option<bool>,

  #[arg(long)]
  pub reset_config: bool
}


fn validate_ip(s: &str) -> Result<String, String> {
  if is_ip::is_ip(s) {
    return Ok(s.to_string());
  }
  Err("IP is not valid".to_owned())
}

fn validate_port(p: &str) -> Result<u16, String> {
  let p = match p.parse::<u16>() {
    Ok(n) => n,
    Err(e) => return Err(format!("Unable to parse as number: {e}"))
  };

  if (1..=0xFFFF).contains(&p) { return Ok(p); } 
  Err("Port is not within valid range: 1 - 65535".to_owned())
}
