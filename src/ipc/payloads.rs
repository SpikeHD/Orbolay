use serde::Deserialize;

use crate::user::{User, UserVoiceState};

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RpcUser {
  pub id: String,
  pub username: String,
  pub global_name: Option<String>,
  pub avatar: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ReadyPayload {
  pub v: i32,
  pub user: Option<RpcUser>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VoiceChannelSelectPayload {
  pub channel_id: Option<String>,
  pub guild_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SelectedVoiceChannelPayload {
  pub id: Option<String>,
  pub guild_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VoiceStateUser {
  pub id: String,
  pub username: String,
  pub global_name: Option<String>,
  pub avatar: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VoiceStateState {
  pub mute: bool,
  pub deaf: bool,
  pub self_mute: bool,
  pub self_deaf: bool,
  pub suppress: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VoiceState {
  pub nick: Option<String>,
  pub mute: bool,
  pub volume: f32,
  pub voice_state: VoiceStateState,
  pub user: VoiceStateUser,
}

impl From<VoiceState> for User {
  fn from(val: VoiceState) -> Self {
    let voice_state = UserVoiceState::from(&val);

    User {
      name: val
        .nick
        .or(val.user.global_name)
        .unwrap_or(val.user.username),
      id: val.user.id,
      avatar: val.user.avatar.unwrap_or_default(),
      voice_state,
      streaming: false,
    }
  }
}

impl From<VoiceState> for UserVoiceState {
  fn from(val: VoiceState) -> Self {
    if val.voice_state.deaf || val.voice_state.self_deaf {
      UserVoiceState::Deafened
    } else if val.voice_state.mute || val.voice_state.self_mute {
      UserVoiceState::Muted
    } else {
      UserVoiceState::NotSpeaking
    }
  }
}

impl From<&VoiceState> for UserVoiceState {
  fn from(val: &VoiceState) -> Self {
    if val.voice_state.deaf || val.voice_state.self_deaf {
      UserVoiceState::Deafened
    } else if val.voice_state.mute || val.voice_state.self_mute {
      UserVoiceState::Muted
    } else {
      UserVoiceState::NotSpeaking
    }
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VoiceConnectionPing {
  pub time: i64,
  pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VoiceConnectionStatusPayload {
  pub state: String,
  pub hostname: Option<String>,
  pub pings: Vec<VoiceConnectionPing>,
  pub average_ping: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct VoiceSettingsUpdatePayload {
  pub deaf: bool,
  pub mute: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SpeakingPayload {
  pub channel_id: String,
  pub user_id: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GetGuildPayload {
  pub id: String,
  pub name: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GetChannelPayload {
  pub id: String,
  pub name: String,
  pub guild_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MessageNotificationInner {
  pub id: Option<String>,
  pub guild_id: Option<String>,
  pub channel_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NotificationCreatePayload {
  pub message: Option<MessageNotificationInner>,
  pub icon_url: Option<String>,
  pub title: String,
  pub body: String,
}
