use freya::prelude::{Signal, SyncStorage, Writable};
use serde::Deserialize;
use serde_json::Value;
use std::{net::{TcpListener, TcpStream}};
use tungstenite::accept;

use crate::{app_state::AppState, log, payloads::ChannelJoinPayload, success, warn};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BridgeMessage {
  pub cmd: String,
  #[serde(flatten)]
  pub data: Value,
}

pub fn create_websocket(port: u16, app_state: Signal<AppState, SyncStorage>) -> Result<(), Box<dyn std::error::Error>> {
  let server = TcpListener::bind(format!("127.0.0.1:{}", port))?;
  success!("Websocket server started on port {}", port);

  for stream in server.incoming() {
    match stream {
      Ok(stream) => {
        log!("Accepted connection");

        std::thread::spawn(move || {
          ws_stream(stream, app_state).expect("Failed to handle stream");
        });
      }
      Err(e) => {
        warn!("Failed to accept connection: {}", e);
      }
    }
  }

  warn!("Server stopped");

  Ok(())
}

fn ws_stream(stream: TcpStream, mut app_state: Signal<AppState, SyncStorage>) -> Result<(), Box<dyn std::error::Error>> {
  let mut websocket = accept(stream)?;

  websocket.get_mut().set_nonblocking(true)?;

  log!("Stream connected");

  loop {
    // Read from the stream
    if let Ok(msg) = websocket.read() {
      if msg.is_empty() || msg.is_close() {
        break;
      }

      let msg = msg.to_string();
      let msg: BridgeMessage = serde_json::from_str(&msg)?;

      log!("Received message: {:?}", msg);

      match msg.cmd.as_str() {
        "CHANNEL_JOINED" => {
          let data = serde_json::from_value::<ChannelJoinPayload>(msg.data)?;
          let mut users = vec![];

          for voice_state in data.states {
            users.push(voice_state.into());
          }

          (*app_state.write()).voice_users = users;
        }
        "VOICE_STATE_UPDATE" => {

        }
        _ => {
          warn!("Unknown command: {}", msg.cmd);
        }
      }
    }
  }
  Ok(())
}
