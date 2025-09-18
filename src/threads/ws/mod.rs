use crate::{Credentials, DEFAULT_CH};
use std::{
    net::{TcpListener, ToSocketAddrs, TcpStream},
    process::exit,
    sync::{
        atomic::{AtomicBool, Ordering}, Arc, mpsc::Sender
    },
    thread::{sleep, spawn, JoinHandle},
    time::Duration,
};

use crossbeam::channel::{Receiver, bounded};
use ringbuf::traits::Consumer;
use tungstenite::{accept_hdr,  http::{Request, Response}, Message, WebSocket};

use super::create_encoder;

pub fn thread(
    mut rx: impl Consumer<Item = f32> + Send + 'static,
    url: impl ToSocketAddrs,
    filename: Arc<String>,
    credentials: Credentials,
    remote_port: Sender<u16>//Arc<RwLock<Option<u16>>>
) -> JoinHandle<()> {
  let server = match TcpListener::bind(url) {
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
            exit(1);
          }
        }
      }
    }
  });

  for stream in server.incoming() {
    match stream {
      Ok(s) => match accept_hdr(s, |req: &Request<()>, res: Response<()>|  {
        let un = req.headers()
          .get("username")
          .map(|u| u.to_str().unwrap());

        let pw = req.headers()
          .get("password")
          .map(|pw| pw.to_str().unwrap());

        if (un, pw) != (Some(&credentials.username), Some(&credentials.password)) {
          eprintln!("Credentials do not match config");
          exit(1)
        }
        
        match req.headers()
          .get("port")
          .map(|h| 
            h.to_str().unwrap().parse::<u16>().unwrap()
          ) {
          Some(port) => remote_port.send(port),
          None => {
            eprintln!("Could not receive remote listening port");
            exit(1);
          }
        };
        Ok(res)
      }) {
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


fn handle_websocket(ws: &mut WebSocket<TcpStream>, rx: &Receiver<Vec<u8>>) {
  loop {
    while let Ok(page) = rx.recv() {
      if let Err(e) = ws.send(Message::Binary(page.into())) {
        eprintln!("Websocket send error: {e}");
        break;
      }
    }
  }
}
