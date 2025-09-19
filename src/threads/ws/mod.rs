use crate::{threads::create_recorder, Credentials, DEFAULT_CH};
use std::{
    path::Path,
    net::{SocketAddr, TcpStream},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

use crossbeam::channel::{Receiver, bounded};
use ringbuf::traits::Consumer;
use tungstenite::{connect, http::Uri, stream::MaybeTlsStream, ClientRequestBuilder, Message, WebSocket};

use super::create_encoder;

pub fn thread(
    mut rx: impl Consumer<Item = f32> + Send + 'static,
    url: SocketAddr,
    filename: Arc<String>,
    credentials: Credentials,
    shutdown: Receiver<()>
) -> JoinHandle<()> {
  let framesize = 960 * DEFAULT_CH;
  let mut opus_frame_buffer = Vec::with_capacity(framesize);
  let (opus_tx, opus_rx) = bounded::<Vec<u8>>(4096 * 32);

  // let (mut opus_tx, opus_rx) = ringbuf::HeapRb::new(256).split();
  let connected = Arc::new(AtomicBool::new(false));
  let shutdown2 = shutdown.clone();

  // Encoding thread
  let encoder_thread = spawn(move || {
    let mut encoder = create_encoder(&filename);
    loop {
      if shutdown2.try_recv().is_ok() { break; }
      if let Some(sample) = rx.try_pop() {
        opus_frame_buffer.push(sample);
      } else {
        sleep(Duration::from_millis(2));
      }

      if opus_frame_buffer.len() == framesize {
        encoder
          .write_float(&opus_frame_buffer)
          .expect("block not a multiple of input channels");
        opus_frame_buffer.clear();
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
  });


  let uri = Uri::builder()
    .scheme("ws")
    .authority(format!("{}:{}", url.ip(), url.port()))
    .path_and_query("/")
    .build()
    .unwrap();

  loop {
    if shutdown.try_recv().is_ok() { break; }
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
          spawn(move || {
            handle_websocket(&mut ws, &opus_rx_receiver);
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
  encoder_thread
}

pub fn rec_thread(
    mut rx: impl Consumer<Item = f32> + Send + 'static,
    url: SocketAddr,
    path: &Path,
    filename: Arc<String>,
    credentials: Credentials,
    shutdown: Receiver<()>
) -> JoinHandle<()> {
  let (opus_tx, opus_rx) = bounded::<Vec<u8>>(4096 * 32);

  // let (mut opus_tx, opus_rx) = ringbuf::HeapRb::new(256).split();
  let connected = Arc::new(AtomicBool::new(false));
  let shutdown2 = shutdown.clone();
  
  let out_path = path.join(filename.clone().to_string());

  // Encoding thread
  let encoder_thread = spawn(move || {
    let mut stream_encoder = create_encoder(&filename);
    let mut local_encoder = create_recorder(&out_path, &filename);
    let framesize = 960 * DEFAULT_CH;
    let mut opus_frame_buffer = Vec::with_capacity(framesize);
    loop {
      if shutdown2.try_recv().is_ok() { break; }
      if let Some(sample) = rx.try_pop() {
        opus_frame_buffer.push(sample);
      } else {
        sleep(Duration::from_millis(2));
      }

      if opus_frame_buffer.len() == framesize {
        local_encoder
          .write_float(&opus_frame_buffer)
          .expect("block not a multiple of input channels");
        stream_encoder
          .write_float(&opus_frame_buffer)
          .expect("block not a multiple of input channels");
        opus_frame_buffer.clear();
        // flush forces encoder to return a page, even if not ready.
        // true is used when realtime streaming is more important than stability.
        if let Some(page) = stream_encoder.get_page(true) {
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
  });


  let uri = Uri::builder()
    .scheme("ws")
    .authority(format!("{}:{}", url.ip(), url.port()))
    .path_and_query("/")
    .build()
    .unwrap();

  loop {
    if shutdown.try_recv().is_ok() { break; }
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
          spawn(move || {
            handle_websocket(&mut ws, &opus_rx_receiver);
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
  encoder_thread
}

fn handle_websocket(ws: &mut WebSocket<MaybeTlsStream<TcpStream>>, rx: &Receiver<Vec<u8>>) {
  loop {
    while let Ok(page) = rx.recv() {
      if let Err(e) = ws.send(Message::Binary(page.into())) {
        eprintln!("Websocket send error: {e}");
        return;
      }
    }
  }
}
