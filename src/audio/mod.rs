
use crate::{
  AUDIO_INTERFACE_NOT_FOUND,
  DEFAULT_INPUT,
  DEFAULT_CH,
  DEFAULT_SR,  
  err::default_not_found,
  Arc, 
  AtomicBool,
  Ordering,
  PathBuf,
};

use std::{
  time::Duration, 
  process::exit,
  thread::sleep
};
use opusenc::{Comments, Encoder, RecommendedTag};
use crossbeam::channel::{Receiver, Sender};
use ringbuf::traits::Consumer;

use cpal::{
    Device, Host,
    traits::{DeviceTrait, HostTrait},
};

/// Searches and matches on the audio interfaces available to the host.
/// Returns an error if unable to access audio input devices, 
/// or if the [`DEFAULT_INPUT`] or any other interface was not available on the host.
pub fn find_audio_device(host: &Host, audio_interface: &str) -> anyhow::Result<Device> {
  let devices = host.input_devices().map_err(|err| {
    anyhow::anyhow!(
      "Could not list input devices: {err}\n\
       Make sure your audio hardware is connected and accessible"
    )
  })?;
  if let Some(dev) = devices
    .filter_map(|d| d.name().ok().map(|n| (d, n)))
    .find(|(_, name)| name == audio_interface)
    .map(|(d, _)| d)
  {
    return Ok(dev);
  }
  if audio_interface == DEFAULT_INPUT {
    Err(anyhow::anyhow!("{}", default_not_found()))
  } else {
    Err(anyhow::anyhow!("{}", AUDIO_INTERFACE_NOT_FOUND))
  }
}

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


pub(crate) fn record_audio(
  shutdown: Arc<AtomicBool>,
  filename: Arc<String>,
  in_rx: &Receiver<f32>,
  path: &PathBuf,
  framesize: usize
) {
  let mut encoder = create_recorder(path, &filename);
  let mut buf = Vec::with_capacity(framesize);
  loop {
    if shutdown.load(Ordering::SeqCst) { break; }
    if let Ok(sample) = in_rx.recv() {
      buf.push(sample);
    }
    if buf.len() == framesize {
      encoder
        .write_float(&buf)
        .expect("block not a multiple of input channels");
      buf.clear();
    }
  }
}

pub(crate) fn encode_audio(
  shutdown: Arc<AtomicBool>,
  filename: Arc<String>,
  in_rx: &Receiver<f32>,
  opus_tx: &Sender<Vec<u8>>,
  framesize: usize
) {
  let mut encoder = create_encoder(&filename);
  let mut buf = Vec::with_capacity(framesize);
  loop {
    if shutdown.load(Ordering::SeqCst) { break; }
    if let Ok(sample) = in_rx.recv() {
      buf.push(sample);
    }
    if buf.len() == framesize {
      encoder
        .write_float(&buf)
        .expect("block not a multiple of input channels");
      buf.clear();
      // flush forces encoder to return a page, even if not ready.
      // true is used when realtime streaming is more important than stability.
      if let Some(page) = encoder.get_page(true) {
        // TODO: NO SOUND IS GETTING THROUGH HERE
        if let Err(e) = opus_tx.send(page.to_vec()) {
          eprint!(
                " \
          Could not append encoded ogg to shared \
          ringbuffer to websocket thread: {e}"
          );
          break;
          // exit(1);
        }
      }
    }
  }
}

/// Fans out the audio stream to (optional) multiple consumers - Broadcast style!
pub(crate) fn audio_capture_loop(shutdown: Arc<AtomicBool>, producer: &mut (impl Consumer<Item = f32> + Send + 'static), consumers: &[Sender<f32>]) {
  loop {
    if shutdown.load(Ordering::SeqCst) { break; }
    if let Some(sample) = producer.try_pop() {
      consumers.iter().for_each(|c| {
        if let Err(e) = c.send(sample) {
          eprintln!("Could not fan out audio stream: {e}")
        }
      });
    } else {
      sleep(Duration::from_millis(2));
    }
  }
}
