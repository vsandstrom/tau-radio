use crate::{DEFAULT_CH, DEFAULT_SR};
use std::{
    fmt::format,
    net::{TcpListener, TcpStream, ToSocketAddrs, UdpSocket},
    path::PathBuf,
    process::exit,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{JoinHandle, sleep, spawn},
    time::Duration,
};

use crossbeam::channel::{Receiver, bounded};
use opusenc::{Comments, Encoder, RecommendedTag};
use ringbuf::traits::Consumer;
use shout::ShoutConn;
use tungstenite::{Message, WebSocket, accept};

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

/// Spawns a thread producing a continuous stream to IceCast host,
/// consuming samples from the audio device chosen and encodes into OggOpus
pub fn icecast_thread(
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
pub fn icecast_rec_thread(
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

pub fn websocket_thread(
    mut rx: impl Consumer<Item = f32> + Send + 'static,
    ip: String,
    port: u16,
    filename: Arc<String>,
) -> JoinHandle<()> {
  let server = match TcpListener::bind(format!("{ip}:{port}")) {
    Ok(tcp) => tcp,
    Err(e) => {
        eprintln!("Unable to bind to address: {e}");
        exit(1)
    }
  };
  let framesize = 960 * DEFAULT_CH;
  let mut opus_frame_buffer = Vec::with_capacity(framesize);
  let (opus_tx, opus_rx) = bounded::<Vec<u8>>(4096 * 32);

  // let (mut opus_tx, opus_rx) = ringbuf::HeapRb::new(256).split();
  let connected = Arc::new(AtomicBool::new(false));

  // Encoding thread
  let encoder_thread = spawn(move || {
    let mut encoder = create_encoder(&filename);

    loop {
      if let Some(sample) = rx.try_pop() {
        opus_frame_buffer.push(sample);
      } else {
        sleep(Duration::from_millis(2));
      }

      if opus_frame_buffer.len() == framesize {
        encoder
          .write_float(&opus_frame_buffer)
          .expect("block not a multiple of input channels");
        // flush forces encoder to return a page, even if not ready.
        // true is used when realtime streaming is more important than stability.
        opus_frame_buffer.clear();
        if let Some(page) = encoder.get_page(true) {
          // TODO: NO SOUND IS GETTING THROUGH HERE
          println!("send");
          if let Err(e) = opus_tx.send(page.to_vec()) {
            eprint!(
                  " \
            Could not append encoded ogg to shared \
            ringbuffer to websocket thread: {e}"
            );
            exit(1);
          }
        }
      }
    }
  });

  for stream in server.incoming() {
    match stream {
      Ok(s) => match accept(s) {
        Ok(mut ws) => {
          if connected.load(Ordering::SeqCst) {
            let _ = ws.close(None);
            continue;
          }

          let connected_inner = connected.clone();
          let opus_rx_receiver = opus_rx.clone();
          spawn(move || {
            handle_websocket(&mut ws, &opus_rx_receiver);
            connected_inner.store(false, Ordering::SeqCst);
          });
        }
        Err(e) => {
          eprintln!("HandshakeError: {e}");
          exit(1);
        }
      },
      Err(_) => todo!(),
    }
  }
  encoder_thread
}

pub fn udp_thread(
    mut rx: impl Consumer<Item = f32> + Send + 'static,
    ip: String,
    port: u16,
    filename: Arc<String>,
) -> JoinHandle<()> {
  let mut socket = match UdpSocket::bind("127.0.0.1:0") {
    Ok(s) => s,
    Err(e) => {
      eprintln!("Could not bind to addr: {e}");
      exit(1)
    }
  };
  let framesize = 960 * DEFAULT_CH;
  let mut opus_frame_buffer = Vec::with_capacity(framesize);
  let (opus_tx, opus_rx) = bounded::<Vec<u8>>(4096 * 32);

  // Encoding thread
  let _ = spawn(move || {
    let mut encoder = create_encoder(&filename);

    loop {
      if let Some(sample) = rx.try_pop() {
        opus_frame_buffer.push(sample);
      } else {
        sleep(Duration::from_millis(2));
      }

      if opus_frame_buffer.len() == framesize {
        encoder
          .write_float(&opus_frame_buffer)
          .expect("block not a multiple of input channels");
        // flush forces encoder to return a page, even if not ready.
        // true is used when realtime streaming is more important than stability.
        opus_frame_buffer.clear();

        if let Some(page) = encoder.get_page(true) {
          if let Err(e) = opus_tx.send(page.to_vec()) {
            eprint!(
                  " \
            Could not append encoded ogg to shared \
            ringbuffer to websocket thread: {e}"
              );
            exit(1);
          }
        }
      }
    }
  });

  // UDP Thread
  spawn(move || {
    handle_udpsocket(&mut socket, &opus_rx, format!("{ip}:{port}"));
  })
}

fn handle_udpsocket(udp: &mut UdpSocket, rx: &Receiver<Vec<u8>>, url: impl ToSocketAddrs) {
  loop {
    while let Ok(page) = rx.recv() {
      println!("sending: {}", page.len());
      if let Err(e) = udp.send_to(&page, &url) {
        eprintln!("UDP socket send error: {e}");
        break;
      }
    }
  }
}

fn handle_websocket(ws: &mut WebSocket<TcpStream>, rx: &Receiver<Vec<u8>>) {
  loop {
    while let Ok(page) = rx.recv() {
      println!("sending: {}", page.len());
      if let Err(e) = ws.send(Message::Binary(page.into())) {
        eprintln!("Websocket send error: {e}");
        break;
      }
    }
  }
}
