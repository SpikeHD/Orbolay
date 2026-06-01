use serde_json::Value;
use std::net::{TcpListener, TcpStream};
use tungstenite::{Message, Utf8Bytes, accept};

use crate::{
  app_state::SharedAppState,
  config::Config,
  error, log,
  payloads::{ChannelJoinPayload, MessageNotificationPayload, UpdatePayload},
  success,
  util::bridge::BridgeMessage,
  warn,
};

pub fn create_websocket(
  port: u16,
  shared: SharedAppState,
  redraw_tx: flume::Sender<()>,
  ws_receiver: flume::Receiver<BridgeMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
  let server = TcpListener::bind(format!("127.0.0.1:{port}"))?;
  success!("Websocket server started on port {}", port);

  for stream in server.incoming() {
    match stream {
      Ok(stream) => {
        log!("Accepted connection");

        let recv = ws_receiver.clone();
        let shared_clone = shared.clone();
        let redraw_clone = redraw_tx.clone();
        std::thread::spawn(
          move || match ws_stream(stream, shared_clone, redraw_clone, recv) {
            Ok(_) => {
              success!("Websocket stream closed");
            }
            Err(e) => {
              error!("Error in websocket stream: {}", e);
            }
          },
        );
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
  shared: SharedAppState,
  redraw_tx: flume::Sender<()>,
  ws_receiver: flume::Receiver<BridgeMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut websocket = accept(stream)?;

  websocket.get_mut().set_nonblocking(true)?;

  log!("Stream connected");

  'outer: loop {
    loop {
      match websocket.read() {
        Ok(msg) if msg.is_close() => {
          log!("Stream closed");
          shared.write().unwrap().voice_users = vec![];
          let _ = redraw_tx.send(());
          break 'outer;
        }
        Ok(msg) if msg.is_empty() => continue,
        Ok(msg) => {
          let msg = msg.to_string();
          let msg: BridgeMessage = serde_json::from_str(&msg)?;
          if let Err(e) = handle_ws_message(&msg, shared.clone(), &redraw_tx) {
            error!("Failed to handle websocket message: {}", e);
          }
        }
        Err(_) => break,
      }
    }

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

    std::thread::sleep(std::time::Duration::from_millis(10));
  }
  Ok(())
}

pub fn handle_ws_message(
  msg: &BridgeMessage,
  shared: SharedAppState,
  redraw_tx: &flume::Sender<()>,
) -> Result<(), Box<dyn std::error::Error>> {
  log!("Received message: {:?}", msg);

  let data = msg.data.clone();
  let mut changed = true;

  match msg.cmd.as_str() {
    "REGISTER_CONFIG" => {
      let user_id = msg
        .data
        .get("userId")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
      let mut state = shared.write().unwrap();
      let existing = state.config.clone();
      let data = serde_json::from_value::<Config>(data).unwrap_or(existing);
      state.config = data;
      state.user_id = user_id;
    }
    "CHANNEL_JOINED" => {
      let data = serde_json::from_value::<ChannelJoinPayload>(data)?;
      let mut state = shared.write().unwrap();
      let user_id = state.user_id.clone();
      let mut users = Vec::with_capacity(data.states.len());

      for voice_state in data.states {
        if voice_state.user_id == user_id {
          state.current_channel = voice_state.channel_id.clone().unwrap_or("0".to_string());
        }
        users.push(voice_state.into());
      }

      state.voice_users = users;
    }
    "VOICE_STATE_UPDATE" => {
      let channel_is_null = msg
        .data
        .get("state")
        .and_then(|v| v.get("channelId"))
        .map(|v| v.is_null())
        .unwrap_or(false);
      let mut data = serde_json::from_value::<UpdatePayload>(msg.data.clone())?;

      let mut state = shared.write().unwrap();

      let should_remove = match &data.state.channel_id {
        Some(channel_id) => channel_id != &state.current_channel,
        None => channel_is_null,
      };

      if should_remove {
        state
          .voice_users
          .retain(|user| user.id != data.state.user_id);
      } else {
        let user = state
          .voice_users
          .iter_mut()
          .find(|user| user.id == data.state.user_id);

        if let Some(user) = user {
          if data.state.streaming.is_none() {
            data.state.streaming = Some(user.streaming);
          }
          user.voice_state = data.state.clone().into();
          user.streaming = data.state.streaming.unwrap_or_default();
        } else {
          state.voice_users.push(data.state.into());
        }
      }
    }
    "CHANNEL_LEFT" => {
      let mut state = shared.write().unwrap();
      state.voice_users = vec![];
      state.current_channel = String::new();
    }
    "MESSAGE_NOTIFICATION" => {
      let mut data = serde_json::from_value::<MessageNotificationPayload>(data)?;
      data.message.timestamp = Some(chrono::Utc::now().timestamp());
      data.message.icon = data.message.icon.replace(".webp", ".png");
      shared.write().unwrap().notify(data.message);
    }
    "STREAMER_MODE" => {
      shared.write().unwrap().is_censor = msg
        .data
        .get("enabled")
        .unwrap_or(&Value::from(false))
        .as_bool()
        .unwrap_or_default();
    }
    _ => {
      warn!("Unknown command: {}", msg.cmd);
      changed = false;
    }
  }

  if changed {
    let _ = redraw_tx.send(());
  }

  Ok(())
}
