use interprocess::local_socket::prelude::*;
use serde_json::Value;

use crate::ipc::{OP_FRAME, ipc_write};

pub fn set_muted(
  stream: &mut LocalSocketStream,
  muted: bool,
) -> Result<(), Box<dyn std::error::Error>> {
  let data = serde_json::json!({ "mute": muted });
  ipc_write(
    stream,
    OP_FRAME,
    &serde_json::to_string(&serde_json::json!({
      "cmd": "SET_VOICE_SETTINGS",
      "args": data,
      "nonce": "SET_VOICE_SETTINGS"
    }))?,
  )?;
  Ok(())
}

pub fn set_deafened(
  stream: &mut LocalSocketStream,
  deafened: bool,
) -> Result<(), Box<dyn std::error::Error>> {
  let data = serde_json::json!({ "deaf": deafened });
  ipc_write(
    stream,
    OP_FRAME,
    &serde_json::to_string(&serde_json::json!({
      "cmd": "SET_VOICE_SETTINGS",
      "args": data,
      "nonce": "SET_VOICE_SETTINGS"
    }))?,
  )?;
  Ok(())
}

pub fn stop_streaming(stream: &mut LocalSocketStream) -> Result<(), Box<dyn std::error::Error>> {
  let data = serde_json::json!({ "streaming": false });
  ipc_write(
    stream,
    OP_FRAME,
    &serde_json::to_string(&serde_json::json!({
      "cmd": "SET_USER_VOICE_SETTINGS",
      "args": data,
      "nonce": "SET_USER_VOICE_SETTINGS"
    }))?,
  )?;
  Ok(())
}

pub fn play_soundboard_sound(
  stream: &mut LocalSocketStream,
  sound_id: &str,
  source_guild_id: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut args = serde_json::json!({ "sound_id": sound_id });
  if let Some(guild_id) = source_guild_id {
    args["guild_id"] = serde_json::json!(guild_id);
  }
  ipc_write(
    stream,
    OP_FRAME,
    &serde_json::to_string(&serde_json::json!({
      "cmd": "PLAY_SOUNDBOARD_SOUND",
      "args": args,
      "nonce": "PLAY_SOUNDBOARD_SOUND"
    }))?,
  )?;
  Ok(())
}

pub fn get_guild(
  stream: &mut LocalSocketStream,
  guild_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  ipc_write(
    stream,
    OP_FRAME,
    &serde_json::to_string(&serde_json::json!({
      "cmd": "GET_GUILD",
      "args": { "guild_id": guild_id },
      "nonce": "GET_GUILD",
    }))?,
  )?;
  Ok(())
}

pub fn get_channel(
  stream: &mut LocalSocketStream,
  channel_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  ipc_write(
    stream,
    OP_FRAME,
    &serde_json::to_string(&serde_json::json!({
      "cmd": "GET_CHANNEL",
      "args": { "channel_id": channel_id },
      "nonce": "GET_CHANNEL",
    }))?,
  )?;
  Ok(())
}

pub fn deep_link_channel(
  stream: &mut LocalSocketStream,
  channel_id: &str,
  guild_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let guild_id = if guild_id.is_empty() { "@me" } else { guild_id };
  ipc_write(
    stream,
    OP_FRAME,
    &serde_json::to_string(&serde_json::json!({
      "cmd": "DEEP_LINK",
      "args": {
        "type": "CHANNEL",
        "params": {
          "guildId": guild_id,
          "channelId": channel_id,
        }
      },
      "nonce": "DEEP_LINK",
    }))?,
  )?;
  Ok(())
}

pub fn select_voice_channel(
  stream: &mut LocalSocketStream,
  channel_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  ipc_write(
    stream,
    OP_FRAME,
    &serde_json::to_string(&serde_json::json!({
      "cmd": "SELECT_VOICE_CHANNEL",
      "args": { "channel_id": channel_id },
      "nonce": "SELECT_VOICE_CHANNEL",
    }))?,
  )?;
  Ok(())
}

pub fn disconnect(stream: &mut LocalSocketStream) -> Result<(), Box<dyn std::error::Error>> {
  let payload = serde_json::json!({ "channel_id": Value::Null });
  ipc_write(
    stream,
    OP_FRAME,
    &serde_json::to_string(&serde_json::json!({
      "cmd": "SELECT_VOICE_CHANNEL",
      "args": payload,
      "nonce": "SELECT_VOICE_CHANNEL"
    }))?,
  )?;
  Ok(())
}
