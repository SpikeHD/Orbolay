use serde::Deserialize;

use crate::{
  AVATAR_CACHE, log,
  user::{User, UserVoiceState},
};

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceState {
  pub user_id: String,
  pub username: Option<String>,
  pub avatar_url: Option<String>,
  pub channel_id: Option<String>,
  pub mute: Option<bool>,
  pub deaf: Option<bool>,
  pub speaking: Option<bool>,
  pub streaming: Option<bool>,
}

impl From<VoiceState> for User {
  fn from(val: VoiceState) -> Self {
    let voice_state = val.clone().into();

    User {
      name: val.username.unwrap_or("Unknown".to_string()),
      id: val.user_id,
      avatar: val.avatar_url.unwrap_or_default(),
      voice_state,
      streaming: val.streaming.unwrap_or_default(),
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct MessageNotificationPayload {
  pub message: MessageNotification,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageNotification {
  pub title: String,
  pub body: String,
  pub icon: String,
  pub channel_id: String,
  pub timestamp: Option<String>,
}

impl MessageNotification {
  pub fn fetch_icon(&self) -> Result<Vec<u8>, ureq::Error> {
    let icon = if self.icon.starts_with("/") {
      format!("https://discord.com{}", self.icon)
    } else {
      self.icon.clone()
    };

    if let Some(img) = AVATAR_CACHE().get(&icon) {
      log!("Cache hit for icon {}", icon);
      return Ok(img.clone());
    }

    log!("Fetching icon from {}", icon);
    let img = ureq::get(&icon).call()?.body_mut().read_to_vec()?;

    (*AVATAR_CACHE.write()).insert(icon.clone(), img.clone());

    Ok(img)
  }
}
