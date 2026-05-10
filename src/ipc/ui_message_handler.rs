use dioxus::prelude::{Signal, SyncStorage};
use interprocess::local_socket::prelude::*;

use freya::prelude::Writable;

use crate::app_state::AppState;
use crate::ipc::setters::{disconnect, set_deafened, set_muted, stop_streaming};
use crate::log;
use crate::util::bridge::BridgeMessage;

pub fn handle_ui_message(
  stream: &mut LocalSocketStream,
  msg: &BridgeMessage,
  app_state: &mut Signal<AppState, SyncStorage>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut state = app_state.write();

  log!("Handling UI message: {:?}", msg);

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
