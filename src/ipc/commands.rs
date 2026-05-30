use std::collections::HashMap;
use std::time::Duration;

use interprocess::TryClone;
#[cfg(unix)]
use interprocess::local_socket::{GenericFilePath, ToFsName, prelude::*};
#[cfg(windows)]
use interprocess::local_socket::{GenericNamespaced, ToNsName, prelude::*};

use crate::app_state::SharedAppState;
use crate::ipc::{
  OP_CLOSE, OP_FRAME, OP_HANDSHAKE, SelectedVoiceChannelPayload, handle_ipc_message,
  handle_ui_message, ipc_read, ipc_write, subscribe_voice_channel, subscribe_voice_global,
};
use crate::log;
use crate::payloads::{MessageNotification, SoundboardSoundPayload};
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
  shared: SharedAppState,
  redraw_tx: &flume::Sender<()>,
) -> Result<(), Box<dyn std::error::Error>> {
  while let Ok(msg) = receiver.try_recv() {
    handle_ui_message(stream, &msg, shared.clone(), redraw_tx)?;
  }

  Ok(())
}

pub fn create_ipc_connection(
  shared: SharedAppState,
  redraw_tx: flume::Sender<()>,
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
    let mut ui_stream = stream.try_clone()?;
    let (tx, rx) = flume::unbounded();
    let ui_shared = shared.clone();
    let ui_redraw = redraw_tx.clone();

    std::thread::spawn(move || {
      loop {
        if let Err(e) =
          drain_ui_messages(&mut ui_stream, &ui_receiver, ui_shared.clone(), &ui_redraw)
        {
          error!("UI message handler failed: {}", e);
          continue;
        }

        if let Ok(()) = rx.try_recv() {
          return;
        }

        std::thread::sleep(Duration::from_millis(10));
      }
    });

    loop {
      match ipc_read(&mut stream) {
        Ok((OP_FRAME, payload)) => {
          if let Ok(msg) = serde_json::from_str::<BridgeMessage>(&payload)
            && let Err(e) = handle_command(&mut stream, &msg, shared.clone(), &redraw_tx)
          {
            error!("Error handling IPC command: {}", e);
          }
        }
        Ok((OP_CLOSE, payload)) => {
          log!("Stream closed: {}", payload);
          shared.write().unwrap().voice_users = vec![];
          let _ = redraw_tx.send(());
          break;
        }
        Err(e) => {
          error!("IPC read error: {}", e);
          shared.write().unwrap().voice_users = vec![];
          let _ = redraw_tx.send(());
          break;
        }
        _ => {}
      }
    }

    tx.send(())?;

    {
      let mut state = shared.write().unwrap();
      state.notify(MessageNotification {
        title: "Disconnected".into(),
        body: "The connection to the Discord client has been closed, retrying in 10 seconds..."
          .into(),
        icon: "".into(),
        ..Default::default()
      });
    }
    let _ = redraw_tx.send(());

    std::thread::sleep(Duration::from_secs(10));
  }
}

pub fn handle_command(
  stream: &mut LocalSocketStream,
  msg: &BridgeMessage,
  shared: SharedAppState,
  redraw_tx: &flume::Sender<()>,
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

      let request = serde_json::json!({
        "cmd": "GET_SOUNDBOARD_SOUNDS",
        "args": {},
        "nonce": "GET_SOUNDBOARD_SOUNDS",
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
      if let Some(data) = serde_json::from_value::<SelectedVoiceChannelPayload>(data)
        .ok()
        .filter(|d| d.id.is_some())
      {
        let channel_id = data.id.unwrap();
        let mut state = shared.write().unwrap();
        state.current_channel = channel_id.clone();
        state.current_guild_id = data.guild_id.unwrap_or_default();
        drop(state);
        subscribe_voice_channel(stream, &channel_id)?;
        let _ = redraw_tx.send(());
      }
    }
    "GET_SOUNDBOARD_SOUNDS" => {
      let data = msg.data.get("data").cloned().unwrap_or_default();
      if let Ok(payload) = serde_json::from_value::<Vec<SoundboardSoundPayload>>(data) {
        let mut by_guild: HashMap<String, Vec<_>> = HashMap::new();
        for sound in payload {
          by_guild
            .entry(sound.guild_id.clone().unwrap_or_default())
            .or_default()
            .push(sound);
        }
        let mut state = shared.write().unwrap();
        for (guild_id, sounds) in by_guild {
          state.soundboard_cache.insert(guild_id, sounds);
        }
        let _ = redraw_tx.send(());
      }
    }
    "DISPATCH" => {
      handle_ipc_message(stream, msg, shared, redraw_tx)?;
    }
    _ => {
      log!("Unhandled command: {}", msg.cmd);
    }
  }

  Ok(())
}
