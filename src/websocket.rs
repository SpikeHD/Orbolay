use freya::prelude::{Readable, Signal, SyncStorage, Writable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::{TcpListener, TcpStream};
use tungstenite::{Message, Utf8Bytes, accept};

use crate::{
  app_state::AppState,
  config::Config,
  error, log,
  payloads::{ChannelJoinPayload, MessageNotificationPayload, UpdatePayload},
  success, warn,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BridgeMessage {
  pub cmd: String,
  #[serde(flatten)]
  pub data: Value,
}

pub fn create_websocket(
  port: u16,
  app_state: Signal<AppState, SyncStorage>,
  ws_receiver: flume::Receiver<BridgeMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
  let server = TcpListener::bind(format!("127.0.0.1:{port}"))?;
  success!("Websocket server started on port {}", port);

  for stream in server.incoming() {
    match stream {
      Ok(stream) => {
        log!("Accepted connection");

        let recv = ws_receiver.clone();
        std::thread::spawn(move || match ws_stream(stream, app_state, recv) {
          Ok(_) => {
            success!("Websocket stream closed");
          }
          Err(e) => {
            error!("Error in websocket stream: {}", e);
          }
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
  ws_receiver: flume::Receiver<BridgeMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut websocket = accept(stream)?;

  websocket.get_mut().set_nonblocking(true)?;

  log!("Stream connected");

  loop {
    // TODO find a better way to do this
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Read from the stream
    if let Ok(msg) = websocket.read() {
      if msg.is_close() {
        log!("Stream closed");
        // Safe to assume there is only one websocket client connected, and we can wipe state
        app_state.write().voice_users = vec![];
        break;
      }

      if msg.is_empty() {
        continue;
      }

      let msg = msg.to_string();
      let msg: BridgeMessage = serde_json::from_str(&msg)?;

      log!("Received message: {:?}", msg);

      match msg.cmd.as_str() {
        "REGISTER_CONFIG" => {
          let user_id = msg
            .data
            .get("userId")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
          let mut data = serde_json::from_value::<Config>(msg.data).unwrap_or_default();

          // Whether the config was valid or not, we still want the user_id to be set
          data.user_id = user_id;

          app_state.write().config = data;
        }
        "CHANNEL_JOINED" => {
          let data = serde_json::from_value::<ChannelJoinPayload>(msg.data)?;
          let mut users = vec![];

          for voice_state in data.states {
            users.push(voice_state.clone().into());

            if voice_state.user_id == app_state.read().config.user_id {
              // Set the current channel to the one we joined
              app_state.write().current_channel =
                voice_state.channel_id.clone().unwrap_or("0".to_string());
            }
          }

          app_state.write().voice_users = users;
        }
        "VOICE_STATE_UPDATE" => {
          let current_channel = app_state.read().current_channel.clone();
          let mut state = app_state.write();
          let mut data = serde_json::from_value::<UpdatePayload>(msg.data)?;

          // Check if the user is leaving the current channel
          let is_leaving = match &data.state.channel_id {
            Some(id) => id != &current_channel,
            None => false, // If ID is missing, assume they are still in the channel but updating status
          };

          if is_leaving {
            state.voice_users.retain(|u| u.id != data.state.user_id);
          } else {
            // Attempt to find and update the user
            let existing_index = state.voice_users.iter().position(|u| u.id == data.state.user_id);

            if let Some(index) = existing_index {
              let user = &mut state.voice_users[index];

              // Preserve streaming state if not provided in update
              if data.state.streaming.is_none() {
                data.state.streaming = Some(user.streaming);
              }

              // Update the user's voice state (Speaking, Muted, etc.)
              user.voice_state = data.state.clone().into();
              user.streaming = data.state.streaming.unwrap_or_default();
            } else {
              // If they aren't in the list but are in our channel, add them
              state.voice_users.push(data.state.into());
            }
          }
        }
        "CHANNEL_LEFT" => {
          // User left the channel, no more need for list
          app_state.write().voice_users = vec![];
          app_state.write().current_channel = String::new();
        }
        "MESSAGE_NOTIFICATION" => {
          let mut data = serde_json::from_value::<MessageNotificationPayload>(msg.data)?;
          data.message.timestamp = Some(chrono::Utc::now().timestamp().to_string());
          data.message.icon = data.message.icon.replace(".webp", ".png");
          let messages_len = app_state.read().messages.len();

          // Keep the last 3 elements
          if messages_len > 3 {
            app_state.write().messages.drain(0..messages_len - 3);
          }

          app_state.write().messages.push(data.message);
        }
        "STREAMER_MODE" => {
          app_state.write().is_censor = msg
            .data
            .get("enabled")
            .unwrap_or(&Value::from(false))
            .as_bool()
            .unwrap_or_default();
        }
        _ => {
          warn!("Unknown command: {}", msg.cmd);
        }
      }
    } else {
      // Try to retrieve something to send to the websocket
      if let Ok(msg) = ws_receiver.try_recv() {
        let msg = serde_json::to_string(&msg)?;
        log!("Sending message to websocket: {:?}", msg);
        websocket
          .write(Message::Text(Utf8Bytes::from(msg)))
          .unwrap_or_else(|_| error!("Failed to send message to websocket, socket closed?"));
        websocket
          .flush()
          .unwrap_or_else(|_| error!("Failed to flush message to websocket, socket closed?"));
      }
    }
  }
  Ok(())
}
