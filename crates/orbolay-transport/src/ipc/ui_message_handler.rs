use interprocess::local_socket::prelude::*;

use orbolay_core::{app_state::AppHandle, user::UserVoiceState, util::bridge::BridgeMessage};

use crate::ipc::setters::{
  deep_link_channel, disconnect, play_soundboard_sound, select_voice_channel, set_deafened,
  set_muted, set_user_volume, stop_streaming,
};
use orbolay_logging::log;

pub fn handle_ui_message(
  stream: &mut LocalSocketStream,
  msg: &BridgeMessage,
  app: AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
  log!("Handling UI message: {:?}", msg);

  match msg.cmd.as_str() {
    "TOGGLE_MUTE" => {
      let muted = app.read(|state| {
        state
          .voice_users
          .iter()
          .find(|user| user.id == state.user_id)
          .map(|user| user.voice_state == UserVoiceState::Muted)
          .unwrap_or(false)
      });
      set_muted(stream, !muted)?;
      return Ok(());
    }
    "TOGGLE_DEAF" => {
      let deafened = app.read(|state| {
        state
          .voice_users
          .iter()
          .find(|user| user.id == state.user_id)
          .map(|user| user.voice_state == UserVoiceState::Deafened)
          .unwrap_or(false)
      });
      set_deafened(stream, !deafened)?;
      return Ok(());
    }
    "OPEN_CHANNEL" => {
      let channel_id = msg
        .data
        .get("channel_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
      let guild_id = msg
        .data
        .get("guild_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
      deep_link_channel(stream, &channel_id, &guild_id)?;
      return Ok(());
    }
    "ACCEPT_CALL" => {
      let channel_id = msg
        .data
        .get("channel_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
      app.update(|state| {
        state
          .messages
          .retain(|m| m.channel_id.as_deref() != Some(channel_id.as_str()));
      });
      select_voice_channel(stream, &channel_id)?;
      return Ok(());
    }
    "DISCONNECT" => {
      disconnect(stream)?;
      app.update(|state| {
        state.current_channel = String::new();
        state.voice_users.clear();
      });
    }
    "STOP_STREAM" => {
      stop_streaming(stream)?;
      return Ok(());
    }
    "PLAY_SOUNDBOARD_SOUND" => {
      let sound_id = msg
        .data
        .get("sound_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
      let source_guild_id = msg
        .data
        .get("source_guild_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
      play_soundboard_sound(stream, &sound_id, source_guild_id.as_deref())?;
      return Ok(());
    }
    "SET_USER_VOLUME" => {
      let user_id = msg
        .data
        .get("user_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
      let volume = msg
        .data
        .get("volume")
        .and_then(|v| v.as_f64())
        .unwrap_or(100.);
      set_user_volume(stream, &user_id, volume)?;
    }
    _ => {
      log!("Unknown UI command: {}", msg.cmd);
    }
  }

  Ok(())
}
