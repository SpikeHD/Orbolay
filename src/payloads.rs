use serde::Deserialize;

use crate::user::{User, UserVoiceState};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceState {
  pub user_id: String,
  pub username: Option<String>,
  pub channel_id: Option<String>,
  pub mute: Option<bool>,
  pub deaf: Option<bool>,
  pub speaking: Option<bool>,
}

impl From<VoiceState> for User {
  fn from(val: VoiceState) -> Self {
    let voice_state = val.clone().into();

    User {
      name: val.username.unwrap_or("Unknown".to_string()),
      id: val.user_id,
      // TODO implement
      avatar: vec![],
      voice_state,
    }
  }
}

impl From<VoiceState> for UserVoiceState {
  fn from(val: VoiceState) -> Self {
    match (val.mute, val.deaf, val.speaking) {
      (_, Some(true), _) => UserVoiceState::Deafened,
      (Some(true), _, _) => UserVoiceState::Muted,
      (_, _, Some(true)) => UserVoiceState::Speaking,
      _ => UserVoiceState::NotSpeaking,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ChannelJoinPayload {
  pub states: Vec<VoiceState>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UpdatePayload {
  pub state: VoiceState,
}
