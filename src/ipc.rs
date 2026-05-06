use crate::payloads::VoiceState;
use crate::util::discord_auth::{build_rpc_authenticate_request, extract_auth_code};
use crate::{
  app_state::AppState, error, log, success, util::discord_auth::build_rpc_authorize_request,
  websocket::BridgeMessage,
};
use dioxus::signals::{Signal, SyncStorage};
use freya::prelude::Writable;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

// IPC opcodes
const OP_HANDSHAKE: u32 = 0;
const OP_FRAME: u32 = 1;
const OP_CLOSE: u32 = 2;

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

fn ipc_write(
  stream: &mut UnixStream,
  opcode: u32,
  payload: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let payload_bytes = payload.as_bytes();
  let len = payload_bytes.len() as u32;
  let mut header = [0u8; 8];
  header[0..4].copy_from_slice(&opcode.to_le_bytes());
  header[4..8].copy_from_slice(&len.to_le_bytes());
  stream.write_all(&header)?;
  stream.write_all(payload_bytes)?;
  Ok(())
}

fn ipc_read(stream: &mut UnixStream) -> Result<(u32, String), std::io::Error> {
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
        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&payload) {
          if msg["evt"] == "READY" {
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
          let payload = serde_json::to_string(&msg)?;
          log!("Sending message: {}", payload);
          ipc_write(&mut stream, OP_FRAME, &payload)
            .unwrap_or_else(|_| error!("Failed to send message over IPC"));
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
            // We are authorized! Subscribe to events
            success!("Authorized with Discord, subscribing to events");
            for event in &[
              "VOICE_CHANNEL_SELECT",
              "VOICE_SETTINGS_UPDATE",
              "VOICE_CONNECTION_STATUS",
              "NOTIFICATION_CREATE",
            ] {
              if let Err(e) = subscribe(&mut stream, event) {
                error!("Failed to subscribe to {}: {}", event, e);
              } else {
                log!("Subscribed to {}", event);
              }
            }
          } else if msg.cmd == "DISPATCH" {
            handle_ipc_message(&msg, &mut app_state)?;
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
        if let Ok(msg) = ws_receiver.try_recv() {
          let payload = serde_json::to_string(&msg)?;
          log!("Sending message: {}", payload);
          ipc_write(&mut stream, OP_FRAME, &payload)
            .unwrap_or_else(|_| error!("Failed to send message over IPC"));
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

pub fn handle_ipc_message(
  msg: &BridgeMessage,
  app_state: &mut Signal<AppState, SyncStorage>,
) -> Result<(), Box<dyn std::error::Error>> {
  log!("Handling IPC message: {:?}", msg);

  let mut app_state = app_state.write();

  match msg.cmd.as_str() {
    _ => {
      log!("Unknown IPC command: {}", msg.cmd);
    }
  }

  Ok(())
}

fn subscribe(mut stream: &mut UnixStream, event: &str) -> Result<(), Box<dyn std::error::Error>> {
  let subscribe_msg = serde_json::json!({
      "cmd": "SUBSCRIBE",
      "evt": event,
      "args": {},
      "nonce": event,
  });
  ipc_write(&mut stream, OP_FRAME, &subscribe_msg.to_string())?;
  Ok(())
}
