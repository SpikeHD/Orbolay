use freya::prelude::{Signal, SyncStorage, Writable};
use serde::Deserialize;
use serde_json::Value;
use std::net::{TcpListener, TcpStream};
use tungstenite::accept;

use crate::{
  app_state::AppState,
  log,
  payloads::{ChannelJoinPayload, UpdatePayload},
  success,
  warn,
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BridgeMessage {
  pub cmd: String,
  #[serde(flatten)]
  pub data: Value,
}

pub fn create_websocket(
  port: u16,
  app_state: Signal<AppState, SyncStorage>,
) -> Result<(), Box<dyn std::error::Error>> {
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

fn ws_stream(
  stream: TcpStream,
  mut app_state: Signal<AppState, SyncStorage>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut websocket = accept(stream)?;

  websocket.get_mut().set_nonblocking(false)?;

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

          app_state.write().voice_users = users;
        }
        "VOICE_STATE_UPDATE" => {
          let data = serde_json::from_value::<UpdatePayload>(msg.data)?;
          let mut voice_users = app_state().voice_users;
          let user_in_list = voice_users
            .iter_mut()
            .any(|user| user.id == data.state.user_id);

          // If the channel is 0, then they left and we should remove them from the list
          if data.state.channel_id.clone().unwrap_or("1".to_string()) == "0" {
            app_state.write()
              .voice_users
              .retain(|user| user.id != data.state.user_id);
            continue;
          }

          if user_in_list {
            app_state.write()
              .voice_users
              .iter_mut()
              .find(|user| user.id == data.state.user_id)
              .unwrap()
              .voice_state = data.state.clone().into();
          } else {
            // Push them
            app_state.write().voice_users.push(data.state.into());
          }
        }
        "CHANNEL_LEFT" => {
          // User left the channel, no more need for list
          app_state.write().voice_users = vec![];
        }
        _ => {
          warn!("Unknown command: {}", msg.cmd);
        }
      }
    }
  }
  Ok(())
}
