use serde::Deserialize;

use crate::user::{User, UserVoiceState};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceState {
  pub user_id: String,
  pub username: Option<String>,
  pub channel_id: String,
  pub mute: Option<bool>,
  pub deaf: Option<bool>,
  pub speaking: Option<bool>,
}

impl Into<User> for VoiceState {
  fn into(self) -> User {
    User {
      name: self.username.unwrap_or("Unknown".to_string()),
      id: self.user_id,
      // TODO implement
      avatar: vec![],
      voice_state: match (self.mute, self.deaf, self.speaking) {
        (Some(true), _, _) => UserVoiceState::Muted,
        (_, Some(true), _) => UserVoiceState::Deafened,
        (_, _, Some(true)) => UserVoiceState::Speaking,
        _ => UserVoiceState::NotSpeaking,
      },
    }
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ChannelJoinPayload {
  pub states: Vec<VoiceState>,
}
