
use crate::DEFAULT_CH;
use std::{
    net::{ToSocketAddrs, UdpSocket},
    process::exit,
    sync::{
        Arc
    },
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

use crossbeam::channel::{Receiver, bounded};
use ringbuf::traits::Consumer;

use super::create_encoder;

pub fn thread(
    mut rx: impl Consumer<Item = f32> + Send + 'static,
    url: impl ToSocketAddrs + Send + 'static,
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

        if let Some(page) = encoder.get_page(true) 
          && let Err(e) = opus_tx.send(page.to_vec()) {
          eprint!( "Could not append encoded ogg to shared \
                    ringbuffer to websocket thread: {e}");
          exit(1);
        }
      }
    }
  });

  // UDP Thread
  spawn(move || {
    handle_udpsocket(&mut socket, &opus_rx, &url);
  })
}

fn handle_udpsocket(udp: &mut UdpSocket, rx: &Receiver<Vec<u8>>, url: &impl ToSocketAddrs) {
  loop {
    while let Ok(page) = rx.recv() {
      if let Err(e) = udp.send_to(&page, &url) {
        eprintln!("UDP socket send error: {e}");
        break;
      }
    }
  }
}

