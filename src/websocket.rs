use std::net::{TcpListener, TcpStream};

use dioxus::logger::tracing::info;
use tungstenite::accept;

use crate::success;

pub fn create_websocket(port: u16) -> Result<(), Box<dyn std::error::Error>> {
  let server = TcpListener::bind(format!("127.0.0.1:{}", port))?;
  success!("Websocket server started on port {}", port);

  for stream in server.incoming() {
    std::thread::spawn(move || {
      ws_stream(stream.expect("Failed to accept stream")).expect("Failed to handle stream");
    });
  }

  Ok(())
}

fn ws_stream(stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
  let mut websocket = accept(stream)?;

  websocket.get_mut().set_nonblocking(true)?;
  websocket.get_mut().set_nodelay(true)?;

  loop {
    // Read from the stream
    if let Ok(msg) = websocket.read() {
      if msg.is_empty() || msg.is_close() {
        break;
      }

      info!("Received message: {}", msg);
    }
  }
  Ok(())
}