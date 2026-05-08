use dioxus::prelude::{Signal, SyncStorage};

use std::os::unix::net::UnixStream;

use freya::prelude::Writable;

use crate::app_state::AppState;
use crate::error;
use crate::ipc::{
  NotificationCreatePayload, SpeakingPayload, VoiceChannelSelectPayload,
  VoiceConnectionStatusPayload, VoiceSettingsUpdatePayload, VoiceState, subscribe_voice_channel,
  unsubscribe_voice_channel,
};
use crate::log;
use crate::payloads::MessageNotification;

pub fn handle_ipc_message(
  stream: &mut UnixStream,
  msg: &crate::util::bridge::BridgeMessage,
  app_state: &mut Signal<AppState, SyncStorage>,
) -> Result<(), Box<dyn std::error::Error>> {
  let mut state = app_state.write();
  let evt = msg
    .data
    .get("evt")
    .and_then(|v| v.as_str())
    .unwrap_or_default();
  let data = msg.data.get("data").cloned().unwrap_or_default();

  log!("Handling event: {} - {:?}", evt, msg);

  match evt {
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

      if !state.current_channel.is_empty() {
        if let Err(e) = subscribe_voice_channel(stream, &state.current_channel) {
          error!("Failed to subscribe to voice channel events: {}", e);
        }
      } else {
        state.voice_users.clear();
        state.current_channel = String::new();
      }
    }
    "VOICE_STATE_CREATE" | "VOICE_STATE_UPDATE" => {
      let data = serde_json::from_value::<VoiceState>(data)?;
      let user_id = data.user.id.clone();

      if let Some(user) = state.voice_users.iter_mut().find(|user| user.id == user_id) {
        user.name = data
          .nick
          .clone()
          .or(data.user.global_name.clone())
          .unwrap_or(data.user.username.clone());
        user.avatar = data.user.avatar.clone().unwrap_or_default();
        user.voice_state = data.clone().into();
        user.streaming = false;
      } else {
        state.voice_users.push(data.clone().into());
      }
    }
    "SPEAKING_START" => {
      let data = serde_json::from_value::<SpeakingPayload>(data)?;
      if let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == data.user_id)
      {
        user.voice_state = crate::user::UserVoiceState::Speaking;
      }
    }
    "SPEAKING_STOP" => {
      let data = serde_json::from_value::<SpeakingPayload>(data)?;
      if let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == data.user_id)
      {
        user.voice_state = crate::user::UserVoiceState::NotSpeaking;
      }
    }
    "VOICE_STATE_DELETE" => {
      let data = serde_json::from_value::<VoiceState>(data)?;
      state.voice_users.retain(|user| user.id != data.user.id);
    }
    "VOICE_SETTINGS_UPDATE" => {
      let data = serde_json::from_value::<VoiceSettingsUpdatePayload>(data)?;
      let current_user_id = state.config.user_id.clone();
      if let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == current_user_id)
      {
        user.voice_state = if data.deaf {
          crate::user::UserVoiceState::Deafened
        } else if data.mute {
          crate::user::UserVoiceState::Muted
        } else {
          crate::user::UserVoiceState::NotSpeaking
        };
      }
    }
    "VOICE_CONNECTION_STATUS" => {
      let data = serde_json::from_value::<VoiceConnectionStatusPayload>(data)?;
      log!("Avg ping: {}ms", data.average_ping.unwrap_or_default());
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
        timestamp: Some(chrono::Utc::now().timestamp().to_string()),
        icon: data
          .icon_url
          .clone()
          .unwrap_or(String::new())
          .replace(".webp", ".png"),
      };
      let messages_len = state.messages.len();

      if messages_len > 3 {
        state.messages.drain(0..messages_len - 3);
      }

      state.messages.push(notification);
    }
    _ => {
      log!("Unknown IPC command: {}", msg.cmd);
    }
  }

  Ok(())
}
