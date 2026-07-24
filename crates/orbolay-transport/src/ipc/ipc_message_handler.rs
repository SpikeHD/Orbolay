use interprocess::local_socket::prelude::*;

use orbolay_core::{
  app_state::AppHandle, payloads::{Notification, NotificationAction, NotificationKind, ipc::{
    NotificationCreatePayload, ReadyPayload, RpcVoiceState, ScreenshareState, SpeakingPayload, VideoState, VoiceChannelSelectPayload, VoiceConnectionStatusPayload, VoiceSettingsUpdatePayload,
  }}, user::UserVoiceState, util::bridge::BridgeMessage,
};

use crate::ipc::{
  OP_FRAME, ipc_write,
  setters::{get_channel, get_guild},
  subscribe_voice_channel, unsubscribe_voice_channel,
};
use std::sync::Arc;

use orbolay_logging::{error, log, success};

use crate::util::discord_auth::build_rpc_authorize_request;

pub fn handle_ipc_message(
  stream: &mut LocalSocketStream,
  msg: &BridgeMessage,
  app: AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
  let shared = app.shared();
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
      let channel_changed = old_channel != new_channel;

      if channel_changed
        && !old_channel.is_empty()
        && let Err(e) = unsubscribe_voice_channel(stream, &old_channel)
      {
        error!("Failed to unsubscribe from old voice channel events: {}", e);
      }

      if channel_changed {
        state.voice_users.clear();
      }

      state.current_channel = new_channel;
      state.current_guild_id = data.guild_id.unwrap_or_default();
      let channel_id = state.current_channel.clone();
      let guild_id = state.current_guild_id.clone();

      if !channel_id.is_empty() {
        if let Err(e) = subscribe_voice_channel(stream, &channel_id) {
          error!("Failed to subscribe to voice channel events: {}", e);
        }

        if channel_changed {
          let request = serde_json::json!({
            "cmd": "GET_SELECTED_VOICE_CHANNEL",
            "nonce": "GET_SELECTED_VOICE_CHANNEL",
          });
          if let Err(e) = ipc_write(stream, OP_FRAME, &request.to_string()) {
            error!("Failed to request selected voice channel: {}", e);
          }
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
        user.volume = data.volume;
      } else {
        state.voice_users.push(data.into());
      }
    }
    "SCREENSHARE_STATE_UPDATE" => {
      // These are only recieved for the current user, so just set the current user as streaming or not streaming
      let data = serde_json::from_value::<ScreenshareState>(data)?;
      let current_user_id = state.user_id.clone();
      if let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == current_user_id)
      {
        user.streaming = data.active;
      }
    }
    "VIDEO_STATE_UPDATE" => {
      // These are only recieved for the current user, so just set the current user as "camera active" or not
      let data = serde_json::from_value::<VideoState>(data)?;
      let current_user_id = state.user_id.clone();
      if let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == current_user_id)
      {
        user.camera = data.active;
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
      let is_call = message.and_then(|m| m.message_type).is_some_and(|t| t == 3);
      let channel_id = message
        .and_then(|m| m.channel_id.clone())
        .or_else(|| data.channel_id.clone());
      let guild_id = message.and_then(|m| m.guild_id.clone());

      let actions = if is_call {
        channel_id.as_ref().map(|cid| {
          vec![
            NotificationAction {
              label: "Accept".to_string(),
              action: Arc::new({
                let cid = cid.clone();
                let app = app.clone();
                move || {
                  app.send(BridgeMessage {
                    cmd: "ACCEPT_CALL".to_string(),
                    data: serde_json::json!({ "channel_id": &*cid }),
                  });
                }
              }),
              kind: NotificationKind::Primary,
            },
            NotificationAction {
              label: "Open".to_string(),
              action: Arc::new({
                let cid = cid.clone();
                let gid = guild_id.clone().unwrap_or_default();
                let app = app.clone();
                move || {
                  app.send(BridgeMessage {
                    cmd: "OPEN_CHANNEL".to_string(),
                    data: serde_json::json!({ "channel_id": cid, "guild_id": gid }),
                  });
                  app.update(|st| {
                    st.messages
                      .retain(|m| m.channel_id.as_deref() != Some(&cid));
                  });
                }
              }),
              kind: NotificationKind::Secondary,
            },
          ]
        })
      } else {
        None
      };

      let notification = Notification {
        guild_id,
        channel_id,
        message_id: message.and_then(|m| m.id.clone()),
        title: data.title.clone(),
        body: data.body.clone(),
        timestamp: Some(chrono::Utc::now().timestamp()),
        timeout_secs: if is_call { 30 } else { 5 },
        icon: data
          .icon_url
          .clone()
          .unwrap_or_default()
          .replace(".webp", ".png"),
        actions,
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
    app.redraw();
  }

  Ok(())
}
