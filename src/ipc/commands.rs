use dioxus::prelude::{Signal, SyncStorage};
use freya::prelude::Writable;
use std::time::Duration;

use interprocess::TryClone;
#[cfg(unix)]
use interprocess::local_socket::{GenericFilePath, ToFsName, prelude::*};
#[cfg(windows)]
use interprocess::local_socket::{GenericNamespaced, ToNsName, prelude::*};

use crate::app_state::AppState;
use crate::ipc::{
  OP_CLOSE, OP_FRAME, OP_HANDSHAKE, SelectedVoiceChannelPayload, handle_ipc_message,
  handle_ui_message, ipc_read, ipc_write, subscribe_voice_channel, subscribe_voice_global,
};
use crate::log;
use crate::payloads::MessageNotification;
use crate::success;
use crate::util::bridge::BridgeMessage;
use crate::util::discord_auth::{build_rpc_authenticate_request, extract_auth_code};
use crate::{CLIENT_ID, error};

fn try_create_stream() -> Result<LocalSocketStream, Box<dyn std::error::Error>> {
  #[cfg(unix)]
  {
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

        if let Ok(stream) = LocalSocketStream::connect(path.to_fs_name::<GenericFilePath>()?) {
          return Ok(stream);
        }
      }
    }
  }

  #[cfg(windows)]
  {
    for i in 0..10 {
      let path = format!("discord-ipc-{}", i);
      let path = path.to_ns_name::<GenericNamespaced>()?;

      if let Ok(stream) = LocalSocketStream::connect(path) {
        return Ok(stream);
      }
    }
  }

  Err("Could not connect to any Discord IPC socket".into())
}

fn drain_ui_messages(
  stream: &mut LocalSocketStream,
  receiver: &flume::Receiver<BridgeMessage>,
  app_state: &mut Signal<AppState, SyncStorage>,
) -> Result<(), Box<dyn std::error::Error>> {
  while let Ok(msg) = receiver.try_recv() {
    handle_ui_message(stream, &msg, app_state)?;
  }

  Ok(())
}

pub fn create_ipc_connection(
  mut app_state: Signal<AppState, SyncStorage>,
  receiver: flume::Receiver<BridgeMessage>,
) -> Result<(), Box<dyn std::error::Error>> {
  let handshake = serde_json::json!({
    "v": 1,
    "client_id": CLIENT_ID,
  });

  loop {
    let mut stream = match try_create_stream() {
      Ok(s) => s,
      Err(e) => {
        error!("Failed to connect to Discord IPC socket: {}", e);
        std::thread::sleep(Duration::from_secs(10));
        continue;
      }
    };

    ipc_write(&mut stream, OP_HANDSHAKE, &handshake.to_string())?;

    let ui_receiver = receiver.clone();
    let mut ui_app_state = app_state;
    let mut ui_stream = stream.try_clone()?;
    let (tx, rx) = flume::unbounded();

    // Thread to handle UI messages
    std::thread::spawn(move || {
      loop {
        if let Err(e) = drain_ui_messages(&mut ui_stream, &ui_receiver, &mut ui_app_state) {
          error!("UI message handler failed: {}", e);
          continue;
        }

        if let Ok(()) = rx.try_recv() {
          // Kill the thread
          return;
        }

        std::thread::sleep(Duration::from_millis(10));
      }
    });

    loop {
      match ipc_read(&mut stream) {
        Ok((OP_FRAME, payload)) => {
          if let Ok(msg) = serde_json::from_str::<BridgeMessage>(&payload)
            && let Err(e) = handle_command(&mut stream, &msg, &mut app_state)
          {
            error!("Error handling IPC command: {}", e);
          }
        }
        Ok((OP_CLOSE, payload)) => {
          log!("Stream closed: {}", payload);
          app_state.write().voice_users = vec![];
          break;
        }
        Err(e) => {
          error!("IPC read error: {}", e);
          app_state.write().voice_users = vec![];
          break;
        }
        _ => {}
      }
    }

    tx.send(())?;

    app_state.write().notify(MessageNotification {
      title: "Disconnected".into(),
      body: "The connection to the Discord client has been closed, retrying in 10 seconds..."
        .into(),
      icon: "".into(),
      ..Default::default()
    });

    std::thread::sleep(Duration::from_secs(10));
  }
}

pub fn handle_command(
  stream: &mut LocalSocketStream,
  msg: &BridgeMessage,
  app_state: &mut Signal<AppState, SyncStorage>,
) -> Result<(), Box<dyn std::error::Error>> {
  match msg.cmd.as_str() {
    "AUTHENTICATE" => {
      success!("Authorized with Discord, subscribing to events");

      subscribe_voice_global(stream)?;

      let request = serde_json::json!({
        "cmd": "GET_SELECTED_VOICE_CHANNEL",
        "nonce": "GET_SELECTED_VOICE_CHANNEL",
      });
      ipc_write(stream, OP_FRAME, &request.to_string())?;
    }
    "AUTHORIZE" => {
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
          return Err("Failed to extract access token from StreamKit response".into());
        }
      };

      log!("Got StreamKit access token");

      let auth_msg = build_rpc_authenticate_request(&streamkit_code);
      ipc_write(stream, OP_FRAME, &serde_json::to_string(&auth_msg)?)?;

      log!("Sent second stage of auth flow");
    }
    "GET_SELECTED_VOICE_CHANNEL" => {
      let data = msg.data.get("data").cloned().unwrap_or_default();
      let data = serde_json::from_value::<SelectedVoiceChannelPayload>(data).ok();
      if let Some(channel_id) = data.and_then(|d| d.id) {
        app_state.write().current_channel = channel_id.clone();
        subscribe_voice_channel(stream, &channel_id)?;
      }
    }
    "DISPATCH" => {
      handle_ipc_message(stream, msg, app_state)?;
    }
    _ => {
      log!("Unhandled command: {}", msg.cmd);
    }
  }

  Ok(())
}
