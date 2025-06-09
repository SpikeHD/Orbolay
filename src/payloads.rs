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

impl Into<User> for VoiceState {
  fn into(self) -> User {
    let voice_state = self.clone().into();

    User {
      name: self.username.unwrap_or("Unknown".to_string()),
      id: self.user_id,
      // TODO implement
      avatar: vec![],
      voice_state,
    }
  }
}

impl Into<UserVoiceState> for VoiceState {
  fn into(self) -> UserVoiceState {
    match (self.mute, self.deaf, self.speaking) {
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
