#[cfg(unix)]
use interprocess::local_socket::{GenericFilePath, Name};
use interprocess::local_socket::{GenericNamespaced, ToFsName, ToNsName, prelude::*};
use dioxus::prelude::{Signal, SyncStorage};
use freya::prelude::Writable;
#[cfg(unix)]
use interprocess::os::unix::local_socket::FilesystemUdSocket;
use serde_json::Value;

use crate::app_state::AppState;
use crate::error;
use crate::ipc::{
  OP_CLOSE, OP_FRAME, OP_HANDSHAKE, ReadyPayload, SelectedVoiceChannelPayload, handle_ipc_message,
  handle_ui_message, ipc_read, ipc_write, subscribe_voice_channel, subscribe_voice_global,
};
use crate::log;
use crate::success;
use crate::util::bridge::BridgeMessage;
use crate::util::discord_auth::{
  build_rpc_authenticate_request, build_rpc_authorize_request, extract_auth_code,
};

fn get_ipc_path() -> Option<String> {
  let candidates = [
    std::env::var("XDG_RUNTIME_DIR").ok(),
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
    for i in 0..10 {
      let path = format!("{}/discord-ipc-{}", dir, i);
      if std::path::Path::new(&path).exists() {
        return Some(path);
      }
    }
  }
  None
}

pub fn create_ipc_connection(
  mut app_state: Signal<AppState, SyncStorage>,
  receiver: flume::Receiver<BridgeMessage>,
  client_id: String,
) -> Result<(), Box<dyn std::error::Error>> {
  let ipc_path = get_ipc_path().ok_or("Could not find Discord IPC socket")?;
  log!("Connecting to Discord IPC at {}", ipc_path);

  #[cfg(unix)]
  let name = ipc_path.to_fs_name::<GenericFilePath>()?;
  #[cfg(windows)]
  let name = ipc_path.to_ns_name::<GenericNamespaced>()?;

  let mut stream = LocalSocketStream::connect(name)?;

  let handshake = serde_json::json!({
    "v": 1,
    "client_id": client_id,
  });
  ipc_write(&mut stream, OP_HANDSHAKE, &handshake.to_string())?;

  loop {
    match ipc_read(&mut stream) {
      Ok((OP_FRAME, payload)) => {
        log!("Received during handshake: {}", payload);
        if let Ok(msg) = serde_json::from_str::<Value>(&payload)
          && msg["evt"] == "READY"
        {
          if let Some(data) = msg.get("data")
            && let Ok(ready) = serde_json::from_value::<ReadyPayload>(data.clone())
            && let Some(user) = ready.user
          {
            app_state.write().config.user_id = user.id;
          }

          success!("IPC connected and ready");
          break;
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
            success!("Authorized with Discord, subscribing to events");
            if let Err(e) = subscribe_voice_global(&mut stream) {
              error!("Failed to subscribe to global voice events: {}", e);
            }

            let request = serde_json::json!({
              "cmd": "GET_SELECTED_VOICE_CHANNEL",
              "nonce": "GET_SELECTED_VOICE_CHANNEL",
            });
            ipc_write(&mut stream, OP_FRAME, &request.to_string())?;

            continue;
          } else if msg.cmd == "GET_SELECTED_VOICE_CHANNEL" {
            let data = msg.data.get("data").cloned().unwrap_or_default();
            let data = serde_json::from_value::<SelectedVoiceChannelPayload>(data).ok();
            if let Some(channel_id) = data.and_then(|d| d.id) {
              app_state.write().current_channel = channel_id.clone();
              if let Err(e) = subscribe_voice_channel(&mut stream, &channel_id) {
                error!(
                  "Failed to subscribe to existing voice channel events: {}",
                  e
                );
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
