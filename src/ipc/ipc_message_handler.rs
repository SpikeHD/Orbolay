use interprocess::local_socket::prelude::*;

use crate::app_state::SharedAppState;
use crate::ipc::{
  OP_FRAME, ipc_write,
  setters::{get_channel, get_guild},
  subscribe_voice_channel, unsubscribe_voice_channel,
};
use crate::log;
use crate::payloads::MessageNotification;
use crate::payloads::ipc::{
  NotificationCreatePayload, ReadyPayload, RpcVoiceState, SpeakingPayload,
  VoiceChannelSelectPayload, VoiceConnectionStatusPayload, VoiceSettingsUpdatePayload,
};
use crate::user::UserVoiceState;
use crate::util::discord_auth::build_rpc_authorize_request;
use crate::{error, success};

pub fn handle_ipc_message(
  stream: &mut LocalSocketStream,
  msg: &crate::util::bridge::BridgeMessage,
  shared: SharedAppState,
  redraw_tx: &flume::Sender<()>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut state = shared.write().unwrap();
  let evt = msg
    .data
    .get("evt")
    .and_then(|v| v.as_str())
    .unwrap_or_default();
  let data = msg.data.get("data").cloned().unwrap_or_default();

  log!("Handling event: {} - {:?}", evt, msg);

  let mut changed = true;

  match evt {
    "READY" => {
      if let Ok(ready) = serde_json::from_value::<ReadyPayload>(data)
        && let Some(user) = ready.user
      {
        state.user_id = user.id;
      }

      success!("IPC connected and ready");

      let auth_msg = build_rpc_authorize_request();
      ipc_write(stream, OP_FRAME, &serde_json::to_string(&auth_msg)?)?;
    }
    "VOICE_CHANNEL_SELECT" => {
      let data = serde_json::from_value::<VoiceChannelSelectPayload>(data)?;
      let new_channel = data.channel_id.unwrap_or_default();
      let old_channel = state.current_channel.clone();

      if old_channel != new_channel
        && !old_channel.is_empty()
        && let Err(e) = unsubscribe_voice_channel(stream, &old_channel)
      {
        error!("Failed to unsubscribe from old voice channel events: {}", e);
      }

      state.current_channel = new_channel;
      state.current_guild_id = data.guild_id.unwrap_or_default();
      let channel_id = state.current_channel.clone();
      let guild_id = state.current_guild_id.clone();

      if !channel_id.is_empty() {
        if let Err(e) = subscribe_voice_channel(stream, &channel_id) {
          error!("Failed to subscribe to voice channel events: {}", e);
        }

        let request = serde_json::json!({
          "cmd": "GET_SOUNDBOARD_SOUNDS",
          "args": {},
          "nonce": "GET_SOUNDBOARD_SOUNDS",
        });

        if let Err(e) = ipc_write(stream, OP_FRAME, &request.to_string()) {
          error!("Failed to request soundboard sounds: {}", e);
        }
        if let Err(e) = get_channel(stream, &channel_id) {
          error!("Failed to request channel name: {}", e);
        }
        if !guild_id.is_empty()
          && let Err(e) = get_guild(stream, &guild_id)
        {
          error!("Failed to request guild name: {}", e);
        }
      } else {
        state.voice_users.clear();
        state.current_channel = String::new();
      }
    }
    "VOICE_STATE_CREATE" | "VOICE_STATE_UPDATE" => {
      let data = serde_json::from_value::<RpcVoiceState>(data)?;
      let user_id = data.user.id.clone();

      if let Some(user) = state.voice_users.iter_mut().find(|user| user.id == user_id) {
        user.name = data
          .nick
          .as_ref()
          .or(data.user.global_name.as_ref())
          .cloned()
          .unwrap_or_else(|| data.user.username.clone());
        user.avatar = data.user.avatar.clone().unwrap_or_default();
        user.voice_state = UserVoiceState::from(&data);
        user.streaming = false;
      } else {
        state.voice_users.push(data.into());
      }
    }
    "SPEAKING_START" => {
      let data = serde_json::from_value::<SpeakingPayload>(data)?;
      if let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == data.user_id)
      {
        user.voice_state = UserVoiceState::Speaking;
      }
    }
    "SPEAKING_STOP" => {
      let data = serde_json::from_value::<SpeakingPayload>(data)?;
      if let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == data.user_id)
      {
        user.voice_state = UserVoiceState::NotSpeaking;
      }
    }
    "VOICE_STATE_DELETE" => {
      let data = serde_json::from_value::<RpcVoiceState>(data)?;
      state.voice_users.retain(|user| user.id != data.user.id);
    }
    "VOICE_SETTINGS_UPDATE" => {
      let data = serde_json::from_value::<VoiceSettingsUpdatePayload>(data)?;
      let current_user_id = state.user_id.clone();
      if let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == current_user_id)
      {
        user.voice_state = if data.deaf {
          UserVoiceState::Deafened
        } else if data.mute {
          UserVoiceState::Muted
        } else {
          UserVoiceState::NotSpeaking
        };
      }
    }
    "VOICE_CONNECTION_STATUS" => {
      let data = serde_json::from_value::<VoiceConnectionStatusPayload>(data)?;
      log!("Avg ping: {}ms", data.average_ping.unwrap_or_default());
      changed = false;
    }
    "NOTIFICATION_CREATE" => {
      let data = serde_json::from_value::<NotificationCreatePayload>(data)?;
      let message = data.message.as_ref();
      let notification = MessageNotification {
        guild_id: message.and_then(|m| m.guild_id.clone()),
        channel_id: message.and_then(|m| m.channel_id.clone()),
        message_id: message.and_then(|m| m.id.clone()),
        title: data.title.clone(),
        body: data.body.clone(),
        timestamp: Some(chrono::Utc::now().timestamp()),
        icon: data
          .icon_url
          .clone()
          .unwrap_or(String::new())
          .replace(".webp", ".png"),
      };
      state.notify(notification);
    }
    _ => {
      log!("Unknown IPC command: {}", msg.cmd);
      changed = false;
    }
  }

  drop(state);

  if changed {
    let _ = redraw_tx.send(());
  }

  Ok(())
}
