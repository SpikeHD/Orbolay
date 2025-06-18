use freya::prelude::{Readable, Signal, SyncStorage, Writable};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::{TcpListener, TcpStream};
use tungstenite::{Message, Utf8Bytes, accept};

use crate::{
  app_state::AppState,
  config::Config,
  log,
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
        std::thread::spawn(move || {
          ws_stream(stream, app_state, recv).expect("Failed to handle stream");
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
        // Safe to assume there is only one websocket client connected, and wee can wipe state
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
          let data = serde_json::from_value::<Config>(msg.data)?;
          app_state.write().config = data;
        }
        "CHANNEL_JOINED" => {
          let data = serde_json::from_value::<ChannelJoinPayload>(msg.data)?;
          let mut users = vec![];

          for voice_state in data.states {
            users.push(voice_state.into());
          }

          app_state.write().voice_users = users;
        }
        "VOICE_STATE_UPDATE" => {
          let mut data = serde_json::from_value::<UpdatePayload>(msg.data)?;
          let mut voice_users = app_state().voice_users;
          let user_in_list = voice_users
            .iter_mut()
            .any(|user| user.id == data.state.user_id);

          println!(
            "User: {:?}",
            voice_users
              .iter()
              .find(|user| user.id == data.state.user_id)
          );

          // Set "streaming" to the value on the user if it is not included in the payload
          if data.state.streaming.is_none() {
            data.state.streaming = voice_users
              .iter()
              .find(|user| user.id == data.state.user_id)
              .map(|user| user.streaming);
          }

          // If the channel is 0, then they left and we should remove them from the list
          if data.state.channel_id.clone().unwrap_or("1".to_string()) == "0" {
            app_state
              .write()
              .voice_users
              .retain(|user| user.id != data.state.user_id);
            continue;
          }

          if user_in_list {
            let mut state = app_state.write();
            let user = state
              .voice_users
              .iter_mut()
              .find(|user| user.id == data.state.user_id)
              .unwrap();

            user.voice_state = data.state.clone().into();
            user.streaming = data.state.streaming.unwrap_or_default();
          } else {
            // Push them
            app_state.write().voice_users.push(data.state.into());
          }
        }
        "CHANNEL_LEFT" => {
          // User left the channel, no more need for list
          app_state.write().voice_users = vec![];
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
          .expect("Failed to send message to websocket, socket closed?");
        websocket
          .flush()
          .expect("Failed to flush message to websocket, socket closed?");
      }
    }
  }
  Ok(())
}
