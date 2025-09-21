use crate::{threads::create_recorder, Credentials, DEFAULT_CH};
use std::{
    net::{SocketAddr, TcpStream}, path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{sleep, spawn}, time::Duration
};

use crossbeam::channel::{Receiver, bounded, Sender};
use ringbuf::traits::Consumer;
use tungstenite::{connect, http::Uri, stream::MaybeTlsStream, ClientRequestBuilder, Message, WebSocket};

use super::create_encoder;

pub fn thread(
    mut rx: impl Consumer<Item = f32> + Send + 'static,
    url: SocketAddr,
    filename: Arc<String>,
    credentials: Credentials,
    shutdown: Arc<AtomicBool>
) {
  let framesize = 960 * DEFAULT_CH;
  let (opus_tx, opus_rx) = bounded::<Vec<u8>>(4096 * 32);
  let (audio_tx, audio_rx) = bounded::<f32>(4096 * 32);

  let shutdown_clone = shutdown.clone();
  let audio_capture_thread = spawn(move || {
    audio_capture_loop(shutdown_clone, &mut rx, &[audio_tx]);
  });

  let shutdown_clone = shutdown.clone();
  // Encoding thread
  let encoder_thread = spawn(move || {
    encode_audio(shutdown_clone, filename, &audio_rx, &opus_tx, framesize);
  });

  websocket_connect_loop(shutdown, &opus_rx, &url, &credentials);

  if let Err(e) = audio_capture_thread.join() { eprintln!("Audio capture join thread error: {e:?}") };
  if let Err(e) = encoder_thread.join() { eprintln!("Encoder thread join error: {e:?}") };
}

pub fn rec_thread(
    mut rx: impl Consumer<Item = f32> + Send + 'static,
    url: SocketAddr,
    path: &Path,
    filename: Arc<String>,
    credentials: Credentials,
    shutdown: Arc<AtomicBool>
) {
  let framesize = 960 * DEFAULT_CH;
  let (opus_tx, opus_rx) = bounded::<Vec<u8>>(4096 * 32);
  let (encode_tx, encode_rx) = bounded::<f32>(4096 * 32);
  let (record_tx, record_rx) = bounded::<f32>(4096 * 32);
  
  let shutdown_clone = shutdown.clone();
  let audio_capture_thread = spawn(move || {
    audio_capture_loop(shutdown_clone, &mut rx, &[encode_tx, record_tx]);
  });

  let shutdown_clone = shutdown.clone();
  let filename_clone = filename.clone();
  // Encoding thread
  let encoder_thread = spawn(move || {
    encode_audio(shutdown_clone, filename_clone, &encode_rx, &opus_tx, framesize);
  });

  let filename_clone = filename.clone();
  let shutdown_clone = shutdown.clone();
  let out_path = path.join(filename.clone().to_string());
  // Recording thread
  let recorder_thread = spawn(move || {
    record_audio(shutdown_clone, filename_clone, &record_rx, &out_path, framesize);
  });

  websocket_connect_loop(shutdown, &opus_rx, &url, &credentials);

  if let Err(e) = audio_capture_thread.join() { eprintln!("Audio capture join thread error: {e:?}") };
  if let Err(e) = recorder_thread.join() { eprintln!("Recorder thread join error: {e:?}") };
  if let Err(e) = encoder_thread.join() { eprintln!("Encoder thread join error: {e:?}") };
}

fn handle_websocket(shutdown: Arc<AtomicBool>, ws: &mut WebSocket<MaybeTlsStream<TcpStream>>, rx: &Receiver<Vec<u8>>) {
  'outer: loop {
    while let Ok(page) = rx.recv() {
      if shutdown.load(Ordering::SeqCst) { break 'outer; }
      if let Err(e) = ws.send(Message::Binary(page.into())) {
        eprintln!("Websocket send error: {e}");
        return;
      }
    }
  }
}

fn record_audio(
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

fn encode_audio(
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
fn audio_capture_loop(shutdown: Arc<AtomicBool>, producer: &mut (impl Consumer<Item = f32> + Send + 'static), consumers: &[Sender<f32>]) {
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

fn websocket_connect_loop(shutdown: Arc<AtomicBool>, opus_rx: &Receiver<Vec<u8>>, url: &SocketAddr, credentials: &Credentials) {
  let connected = Arc::new(AtomicBool::new(false));
  let uri = Uri::builder()
    .scheme("ws")
    .authority(format!("{}:{}", url.ip(), url.port()))
    .path_and_query("/")
    .build()
    .unwrap();

  loop {
    if shutdown.load(Ordering::SeqCst) { break; }
    if !connected.load(Ordering::SeqCst) {
      let request = ClientRequestBuilder::new(uri.clone())
        .with_header("password", credentials.password.clone())
        .with_header("username", credentials.username.clone())
        .with_header("port", credentials.broadcast_port.to_string());
      match connect(request) {
        Ok((mut ws, _)) => {
          connected.store(true, Ordering::SeqCst);
          let connected_inner = connected.clone();
          let opus_rx_receiver = opus_rx.clone();
          let shutdown_clone = shutdown.clone();
          spawn(move || {
            handle_websocket(shutdown_clone, &mut ws, &opus_rx_receiver);
            connected_inner.store(false, Ordering::SeqCst);
          });
        }
        Err(e) => {
          eprintln!("HandshakeError: {e}");
          sleep(std::time::Duration::from_millis(50));
        }
      }
    } else {
      sleep(std::time::Duration::from_millis(50));
    }
  }

}
