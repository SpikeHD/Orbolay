use crate::ipc_payloads::{
  NotificationCreatePayload, ReadyPayload, SpeakingPayload,
  VoiceChannelSelectPayload, VoiceConnectionStatusPayload, VoiceSettingsUpdatePayload,
  VoiceState,
};
use crate::subscription::{
  subscribe, subscribe_voice_channel, subscribe_voice_global, unsubscribe_voice_channel,
};
use crate::util::bridge::BridgeMessage;
use crate::util::discord_auth::{build_rpc_authenticate_request, extract_auth_code};
use crate::{
  app_state::AppState, error, log, success, util::discord_auth::build_rpc_authorize_request,
};
use dioxus::signals::{Signal, SyncStorage};
use freya::prelude::Writable;
use serde_json::Value;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

// IPC opcodes
pub const OP_HANDSHAKE: u32 = 0;
pub const OP_FRAME: u32 = 1;
pub const OP_CLOSE: u32 = 2;

fn get_ipc_path() -> Option<String> {
  let candidates = [
    std::env::var("XDG_RUNTIME_DIR").ok(),
    // For flatpak
    Some(format!(
      "{}/app/com.discordapp.Discord/",
      std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into())
    )),
    std::env::var("TMPDIR").ok(),
    std::env::var("TMP").ok(),
    std::env::var("TEMP").ok(),
    Some("/tmp".to_string()),
  ];

  for dir in candidates.into_iter().flatten() {
    let path = format!("{}/discord-ipc-0", dir);
    if std::path::Path::new(&path).exists() {
      return Some(path);
    }
  }
  None
}

pub fn ipc_write(
  stream: &mut UnixStream,
  opcode: u32,
  payload: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  log!("Sending payload: {:?}", payload);
  let payload_bytes = payload.as_bytes();
  let len = payload_bytes.len() as u32;
  let mut header = [0u8; 8];
  header[0..4].copy_from_slice(&opcode.to_le_bytes());
  header[4..8].copy_from_slice(&len.to_le_bytes());
  stream.write_all(&header)?;
  stream.write_all(payload_bytes)?;
  Ok(())
}

pub fn ipc_read(stream: &mut UnixStream) -> Result<(u32, String), std::io::Error> {
  let mut header = [0u8; 8];
  stream.read_exact(&mut header)?;
  let opcode = u32::from_le_bytes(header[0..4].try_into().unwrap());
  let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
  let mut payload = vec![0u8; len];
  stream.read_exact(&mut payload)?;
  let s = String::from_utf8(payload)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
  Ok((opcode, s))
}

pub fn create_ipc_connection(
  mut app_state: Signal<AppState, SyncStorage>,
  receiver: flume::Receiver<BridgeMessage>,
  client_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let ipc_path = get_ipc_path().ok_or("Could not find Discord IPC socket")?;
  log!("Connecting to Discord IPC at {}", ipc_path);

  let mut stream = UnixStream::connect(&ipc_path)?;
  stream.set_read_timeout(Some(std::time::Duration::from_millis(50)))?;

  // Send handshake
  let handshake = serde_json::json!({
      "v": 1,
      "client_id": client_id,
  });
  ipc_write(&mut stream, OP_HANDSHAKE, &handshake.to_string())?;

  // Wait for READY
  loop {
    match ipc_read(&mut stream) {
      Ok((OP_FRAME, payload)) => {
        log!("Received during handshake: {}", payload);
        if let Ok(msg) = serde_json::from_str::<Value>(&payload) {
          if msg["evt"] == "READY" {
            if let Some(data) = msg.get("data") {
              if let Ok(ready) = serde_json::from_value::<ReadyPayload>(data.clone()) {
                if let Some(user) = ready.user {
                  app_state.write().config.user_id = user.id;
                }
              }
            }
            success!("IPC connected and ready");
            break;
          }
        }
      }
      Ok((OP_CLOSE, payload)) => {
        return Err(format!("Discord closed connection during handshake: {}", payload).into());
      }
      Err(e)
        if e.kind() == std::io::ErrorKind::WouldBlock
          || e.kind() == std::io::ErrorKind::TimedOut =>
      {
        if let Ok(msg) = receiver.try_recv() {
          handle_ui_message(&mut stream, &msg, &mut app_state)?;
        }
      }
      Err(e) => {
        error!("IPC read error: {}", e);
        app_state.write().voice_users = vec![];
        break;
      }
      _ => {}
    }
  }

  let auth_msg = build_rpc_authorize_request(&client_id);
  ipc_write(&mut stream, OP_FRAME, &serde_json::to_string(&auth_msg)?)?;

  loop {
    // Try to read incoming
    match ipc_read(&mut stream) {
      Ok((OP_FRAME, payload)) => {
        if let Ok(msg) = serde_json::from_str::<BridgeMessage>(&payload) {
          if msg.cmd == "AUTHORIZE" {
            let code = msg
              .data
              .get("data")
              .and_then(|v| v.get("code"))
              .and_then(|v| v.as_str())
              .unwrap_or_default()
              .to_string();

            log!("Received auth code");

            // Now send the token to StreamKit, and get the access token back
            let streamkit_code = match extract_auth_code(&code) {
              Some(token) => token,
              None => {
                error!("Failed to extract access token from StreamKit response");
                continue;
              }
            };

            log!("Got StreamKit access token");

            let auth_msg = build_rpc_authenticate_request(&streamkit_code);
            ipc_write(&mut stream, OP_FRAME, &serde_json::to_string(&auth_msg)?)?;

            log!("Sent second stage of auth flow");
            continue;
          } else if msg.cmd == "AUTHENTICATE" {
            // We are authorized! Subscribe to events that mirror the websocket implementation.
            success!("Authorized with Discord, subscribing to events");
            if let Err(e) = subscribe_voice_global(&mut stream) {
              error!("Failed to subscribe to global voice events: {}", e);
            }

            let current_channel = app_state.write().current_channel.clone();
            if !current_channel.is_empty() {
              if let Err(e) = subscribe_voice_channel(&mut stream, &current_channel) {
                error!("Failed to subscribe to current voice channel events: {}", e);
              }
            }

            continue;
          } else if msg.cmd == "DISPATCH" {
            handle_ipc_message(&mut stream, &msg, &mut app_state)?;
            continue;
          }

          log!("Unhandled bridge message: {:?}", msg);
        }
      }
      Ok((OP_CLOSE, payload)) => {
        log!("Stream closed: {}", payload);
        app_state.write().voice_users = vec![];
        break;
      }
      Err(e)
        if e.kind() == std::io::ErrorKind::WouldBlock
          || e.kind() == std::io::ErrorKind::TimedOut =>
      {
        // Handle UI messages
        if let Ok(msg) = receiver.try_recv() {
          handle_ui_message(&mut stream, &msg, &mut app_state)?;
        }
      }
      Err(e) => {
        error!("IPC read error: {}", e);
        app_state.write().voice_users = vec![];
        break;
      }
      _ => {}
    }
  }

  Ok(())
}

fn handle_ui_message(
  stream: &mut UnixStream,
  msg: &BridgeMessage,
  app_state: &mut Signal<AppState, SyncStorage>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut state = app_state.write();

  match msg.cmd.as_str() {
    "TOGGLE_MUTE" => {
      let muted = state
        .voice_users
        .iter()
        .find(|user| user.id == state.config.user_id)
        .map(|user| user.voice_state == crate::user::UserVoiceState::Muted)
        .unwrap_or(false);
      set_muted(stream, !muted)?;
    }
    "TOGGLE_DEAF" => {
      let deafened = state
        .voice_users
        .iter()
        .find(|user| user.id == state.config.user_id)
        .map(|user| user.voice_state == crate::user::UserVoiceState::Deafened)
        .unwrap_or(false);
      set_deafened(stream, !deafened)?;
    }
    "DISCONNECT" => {
      disconnect(stream)?;
      state.current_channel = String::new();
      state.voice_users.clear();
    }
    "STOP_STREAM" => {
      stop_streaming(stream)?;
    }
    _ => {
      log!("Unknown UI command: {}", msg.cmd);
    }
  }

  Ok(())
}

fn handle_ipc_message(
  stream: &mut UnixStream,
  msg: &BridgeMessage,
  app_state: &mut Signal<AppState, SyncStorage>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut state = app_state.write();
  let evt = msg
    .data
    .get("evt")
    .and_then(|v| v.as_str())
    .unwrap_or_default();
  let data = msg.data.get("data").cloned().unwrap_or_default();

  log!("Handling event: {} - {:?}", evt, msg);

  match evt {
    "VOICE_CHANNEL_SELECT" => {
      let data = serde_json::from_value::<VoiceChannelSelectPayload>(data)?;
      let new_channel = data.channel_id.unwrap_or_default();
      let old_channel = state.current_channel.clone();

      if old_channel != new_channel && !old_channel.is_empty() {
        if let Err(e) = unsubscribe_voice_channel(stream, &old_channel) {
          error!("Failed to unsubscribe from old voice channel events: {}", e);
        }
      }

      state.current_channel = new_channel;

      if !state.current_channel.is_empty() {
        if let Err(e) = subscribe_voice_channel(stream, &state.current_channel) {
          error!("Failed to subscribe to voice channel events: {}", e);
        }
      }
    }
    "VOICE_STATE_CREATE" | "VOICE_STATE_UPDATE" => {
      let data = serde_json::from_value::<VoiceState>(data)?;
      let user_id = data.user.id.clone();

      if let Some(user) = state.voice_users.iter_mut().find(|user| user.id == user_id) {
        user.name = data
          .nick
          .clone()
          .or(data.user.global_name.clone())
          .unwrap_or(data.user.username.clone());
        user.avatar = data.user.avatar.clone().unwrap_or_default();
        user.voice_state = data.clone().into();
        user.streaming = false;
      } else {
        state.voice_users.push(data.clone().into());
      }
    }
    "SPEAKING_START" => {
      let data = serde_json::from_value::<SpeakingPayload>(data)?;
      if let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == data.user_id)
      {
        user.voice_state = crate::user::UserVoiceState::Speaking;
      }
    }
    "SPEAKING_STOP" => {
      let data = serde_json::from_value::<SpeakingPayload>(data)?;
      if let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == data.user_id)
      {
        user.voice_state = crate::user::UserVoiceState::NotSpeaking;
      }
    }
    "VOICE_STATE_DELETE" => {
      let data = serde_json::from_value::<VoiceState>(data)?;
      state.voice_users.retain(|user| user.id != data.user.id);
    }
    "VOICE_SETTINGS_UPDATE" => {
      let data = serde_json::from_value::<VoiceSettingsUpdatePayload>(data)?;
      let current_user_id = state.config.user_id.clone();
      if let Some(user) = state.voice_users.iter_mut().find(|user| user.id == current_user_id) {
        user.voice_state = if data.deaf {
          crate::user::UserVoiceState::Deafened
        } else if data.mute {
          crate::user::UserVoiceState::Muted
        } else {
          crate::user::UserVoiceState::NotSpeaking
        };
      }
    }
    "VOICE_CONNECTION_STATUS" => {
      let data = serde_json::from_value::<VoiceConnectionStatusPayload>(data)?;
      log!("Avg ping: {}ms", data.average_ping.unwrap_or_default());
    }
    "NOTIFICATION_CREATE" => {
      let data = serde_json::from_value::<NotificationCreatePayload>(data)?;
      let mut notification = data.message;
      notification.timestamp = Some(chrono::Utc::now().timestamp().to_string());
      notification.icon = data
        .icon_url
        .clone()
        .unwrap_or(notification.icon)
        .replace(".webp", ".png");
      let messages_len = state.messages.len();

      if messages_len > 3 {
        state.messages.drain(0..messages_len - 3);
      }

      state.messages.push(notification);
    }
    _ => {
      log!("Unknown IPC command: {}", msg.cmd);
    }
  }

  Ok(())
}

fn set_muted(
  stream: &mut UnixStream,
  muted: bool,
) -> Result<(), Box<dyn std::error::Error>> {
  let data = serde_json::json!({ "mute": muted});
  ipc_write(stream, OP_FRAME, &serde_json::to_string(&serde_json::json!({
    "cmd": "SET_VOICE_SETTINGS",
    "args": data,
    "nonce": "SET_VOICE_SETTINGS"
  }))?)?;
  Ok(())
}

fn set_deafened(
  stream: &mut UnixStream,
  deafened: bool,
) -> Result<(), Box<dyn std::error::Error>> {
  let data = serde_json::json!({ "deaf": deafened});
  ipc_write(stream, OP_FRAME, &serde_json::to_string(&serde_json::json!({
    "cmd": "SET_VOICE_SETTINGS",
    "args": data,
    "nonce": "SET_VOICE_SETTINGS"
  }))?)?;
  Ok(())
}

fn stop_streaming(
  stream: &mut UnixStream,
) -> Result<(), Box<dyn std::error::Error>> {
  let data = serde_json::json!({ "streaming": false });
  // TODO
  Ok(())
}

fn disconnect(
  stream: &mut UnixStream,
) -> Result<(), Box<dyn std::error::Error>> {
  let payload = serde_json::json!({ "channel_id": Value::Null });
  ipc_write(stream, OP_FRAME, &serde_json::to_string(&serde_json::json!({
    "cmd": "VOICE_CHANNEL_SELECT",
    "args": payload,
    "nonce": "VOICE_CHANNEL_SELECT"
  }))?)?;
  Ok(())
}
