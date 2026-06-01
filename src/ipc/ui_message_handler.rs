use interprocess::local_socket::prelude::*;

use crate::app_state::SharedAppState;
use crate::ipc::setters::{
  disconnect, play_soundboard_sound, set_deafened, set_muted, stop_streaming,
};
use crate::log;
use crate::util::bridge::BridgeMessage;

pub fn handle_ui_message(
  stream: &mut LocalSocketStream,
  msg: &BridgeMessage,
  shared: SharedAppState,
  redraw_tx: &flume::Sender<()>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut state = shared.write().unwrap();

  log!("Handling UI message: {:?}", msg);

  let mut changed = true;

  match msg.cmd.as_str() {
    "TOGGLE_MUTE" => {
      let muted = state
        .voice_users
        .iter()
        .find(|user| user.id == state.config.user_id)
        .map(|user| user.voice_state == crate::user::UserVoiceState::Muted)
        .unwrap_or(false);
      drop(state);
      set_muted(stream, !muted)?;
      return Ok(()); // IPC will send back a state update
    }
    "TOGGLE_DEAF" => {
      let deafened = state
        .voice_users
        .iter()
        .find(|user| user.id == state.config.user_id)
        .map(|user| user.voice_state == crate::user::UserVoiceState::Deafened)
        .unwrap_or(false);
      drop(state);
      set_deafened(stream, !deafened)?;
      return Ok(());
    }
    "DISCONNECT" => {
      disconnect(stream)?;
      state.current_channel = String::new();
      state.voice_users.clear();
    }
    "STOP_STREAM" => {
      drop(state);
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
      drop(state);
      play_soundboard_sound(stream, &sound_id, source_guild_id.as_deref())?;
      return Ok(());
    }
    _ => {
      log!("Unknown UI command: {}", msg.cmd);
      changed = false;
    }
  }

  drop(state);

  if changed {
    let _ = redraw_tx.send(());
  }

  Ok(())
}
