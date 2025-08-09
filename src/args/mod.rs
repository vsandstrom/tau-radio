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
  #[arg(long)]
  pub url: Option<String>,

  /// IceCast server port
  #[arg(long)]
  pub port: Option<u16>,

  /// Optional custom filename of local copy
  #[arg(long)]
  pub file: Option<String>,

  /// Disables the local recording of stream
  #[arg(long)]
  pub no_recording: bool,

  #[arg(long)]
  pub reset_config: bool
}
