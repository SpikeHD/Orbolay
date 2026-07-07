use std::collections::HashMap;
use std::time::Duration;

use interprocess::TryClone;
#[cfg(unix)]
use interprocess::local_socket::{GenericFilePath, ToFsName, prelude::*};
#[cfg(windows)]
use interprocess::local_socket::{GenericNamespaced, ToNsName, prelude::*};

use orbolay_core::{
  app_state::AppHandle,
  payloads::ipc::{
    GetChannelPayload, GetGuildPayload, GetUserPayload, SelectedVoiceChannelPayload,
  },
  payloads::{Notification, SoundboardSoundPayload},
  util::bridge::BridgeMessage,
};

use crate::ipc::{
  OP_CLOSE, OP_FRAME, OP_HANDSHAKE, handle_ipc_message, handle_ui_message, ipc_read, ipc_write,
  setters::{get_channel, get_guild},
  subscribe_voice_channel, subscribe_voice_global,
};
use orbolay_logging::{error, log, success};

use crate::util::discord_auth::{build_rpc_authenticate_request, extract_auth_code};
use crate::CLIENT_ID;

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
  app: AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
  while let Ok(msg) = receiver.try_recv() {
    handle_ui_message(stream, &msg, app.clone())?;
  }

  Ok(())
}

pub fn create_ipc_connection(
  app: AppHandle,
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
    let ui_app = app.clone();

    std::thread::spawn(move || {
      loop {
        if let Err(e) = drain_ui_messages(&mut ui_stream, &ui_receiver, ui_app.clone()) {
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
            && let Err(e) = handle_command(&mut stream, &msg, app.clone())
          {
            error!("Error handling IPC command: {}", e);
          }
        }
        Ok((OP_CLOSE, payload)) => {
          log!("Stream closed: {}", payload);
          app.update(|state| state.voice_users = vec![]);
          break;
        }
        Err(e) => {
          error!("IPC read error: {}", e);
          app.update(|state| state.voice_users = vec![]);
          break;
        }
        _ => {}
      }
    }

    tx.send(())?;

    app.update(|state| {
      state.notify(Notification {
        title: "Disconnected".into(),
        body: "The connection to the Discord client has been closed, retrying in 10 seconds..."
          .into(),
        icon: "".into(),
        ..Default::default()
      });
    });

    std::thread::sleep(Duration::from_secs(10));
  }
}

pub fn handle_command(
  stream: &mut LocalSocketStream,
  msg: &BridgeMessage,
  app: AppHandle,
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

      let user_id = app.read(|state| state.user_id.clone());
      let request = serde_json::json!({
        "cmd": "GET_USER",
        "args": { "id": user_id },
        "nonce": "GET_USER",
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
        let channel_id = data.id.clone().unwrap_or_default();
        let guild_id = data.guild_id.clone().unwrap_or_default();
        let voice_users = data.voice_states.into_iter().map(Into::into).collect();
        app.update(|state| {
          state.current_channel = channel_id.clone();
          state.current_guild_id = guild_id.clone();
          state.voice_users = voice_users;
        });

        subscribe_voice_channel(stream, &channel_id)?;
        get_channel(stream, &channel_id)?;
        if !guild_id.is_empty() {
          get_guild(stream, &guild_id)?;
        }
      }
    }
    "GET_GUILD" => {
      let data = msg.data.get("data").cloned().unwrap_or_default();
      if let Ok(payload) = serde_json::from_value::<GetGuildPayload>(data) {
        app.update(|state| {
          state.guild_names.insert(payload.id, payload.name);
        });
      }
    }
    "GET_CHANNEL" => {
      let data = msg.data.get("data").cloned().unwrap_or_default();
      if let Ok(payload) = serde_json::from_value::<GetChannelPayload>(data) {
        app.update(|state| {
          state.channel_names.insert(payload.id, payload.name);
        });
      }
    }
    "GET_USER" => {
      let data = msg.data.get("data").cloned().unwrap_or_default();
      if let Ok(payload) = serde_json::from_value::<GetUserPayload>(data) {
        app.update(|state| state.premium_type = payload.premium_type);
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
        let known_guilds: Vec<String> = app.read(|state| {
          by_guild
            .keys()
            .filter(|id| !id.is_empty() && !state.guild_names.contains_key(*id))
            .cloned()
            .collect()
        });
        for guild_id in known_guilds {
          if let Err(e) = get_guild(stream, &guild_id) {
            error!("Failed to fetch guild name for {}: {}", guild_id, e);
          }
        }
        app.update(|state| {
          for (guild_id, sounds) in by_guild {
            state.soundboard_cache.insert(guild_id, sounds);
          }
        });
      }
    }
    "DISPATCH" => {
      handle_ipc_message(stream, msg, app.clone())?;
    }
    _ => {
      log!("Unhandled command: {}", msg.cmd);
    }
  }

  Ok(())
}
