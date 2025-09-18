pub mod ws;
pub mod udp;
pub mod icecast;

use crate::{DEFAULT_CH, DEFAULT_SR};
use std::{
    path::PathBuf,
    process::exit,
};

use opusenc::{Comments, Encoder, RecommendedTag};

fn create_encoder(filename: &str) -> Encoder {
  Encoder::create_pull(
    Comments::create()
        .add(RecommendedTag::Title, filename.to_string())
        .unwrap(),
    DEFAULT_SR,
    DEFAULT_CH,
    opusenc::MappingFamily::MonoStereo,
  )
  .unwrap_or_else(|err| {
    eprintln!("Could not create new realtime .ogg encoder: {err}");
    exit(1)
  })
}

fn create_recorder(path: &PathBuf, filename: &str) -> Encoder {
  Encoder::create_file(
    path,
    // filename.clone().as_str(),
    Comments::create()
      .add(RecommendedTag::Title, filename.to_string())
      .unwrap(),
    DEFAULT_SR,
    DEFAULT_CH,
    opusenc::MappingFamily::MonoStereo,
  )
  .unwrap_or_else(|err| {
    eprintln!("Could not create new local .ogg file: {err}");
    exit(1)
  })
}


