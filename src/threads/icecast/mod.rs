use crate::DEFAULT_CH;

use std::{
    path::PathBuf,
    process::exit,
    sync::Arc,
    thread::{sleep, spawn, JoinHandle},
};

use ringbuf::traits::Consumer;
use shout::ShoutConn;
use super::{create_encoder, create_recorder};

/// Spawns a thread producing a continuous stream to IceCast host,
/// consuming samples from the audio device chosen and encodes into OggOpus
pub fn thread(
    icecast: ShoutConn,
    mut rx: impl Consumer<Item = f32> + Send + 'static,
    filename: Arc<String>,
) -> JoinHandle<()> {
  spawn(move || {
    let mut encoder = create_encoder(&filename);

    let framesize = 960 * DEFAULT_CH;
    let mut opus_frame_buffer: Vec<f32> = Vec::with_capacity(framesize);
    loop {
      if let Some(sample) = rx.try_pop() {
        opus_frame_buffer.push(sample);
      } else {
        // If no samples are available, let CPU breath
        sleep(std::time::Duration::from_millis(2));
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
}


/// Spawns a thread producing a continuous stream to IceCast host,
/// consuming samples from the audio device chosen and encodes into OggOpus.
/// - Saves recording to local file.
pub fn rec_thread(
    icecast: ShoutConn,
    mut rx: impl Consumer<Item = f32> + Send + 'static,
    path: &PathBuf,
    filename: Arc<String>,
) -> JoinHandle<()> {
  if !path.exists() {
    std::fs::create_dir_all(path)
      .map_err(|e| {
        eprintln!("Could not create directory for recordings: {e}");
        exit(1);
      })
      .unwrap()
  }

  let out_path = path.join(filename.clone().to_string());

  spawn(move || {
    let mut local_encoder = create_recorder(&out_path, &filename);
    let mut stream_encoder = create_encoder(&filename);
    let framesize = 960 * DEFAULT_CH;
    let mut opus_frame_buffer = Vec::with_capacity(framesize);
    loop {
      if let Some(sample) = rx.try_pop() {
        opus_frame_buffer.push(sample);
      } else {
        sleep(std::time::Duration::from_millis(2));
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
}
