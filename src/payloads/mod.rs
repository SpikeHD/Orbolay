use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageNotification {
  pub title: String,
  pub body: String,
  pub icon: String,
  pub guild_id: Option<String>,
  pub channel_id: Option<String>,
  pub message_id: Option<String>,
  #[serde(default, skip_deserializing)]
  pub timestamp: Option<i64>,
}

impl Default for MessageNotification {
  fn default() -> Self {
    Self {
      title: String::new(),
      body: String::new(),
      icon: String::new(),
      guild_id: None,
      channel_id: None,
      message_id: None,
      timestamp: Some(chrono::Utc::now().timestamp()),
    }
  }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SoundboardSoundPayload {
  pub sound_id: String,
  pub name: String,
  pub volume: f64,
  // Custom emoji
  pub emoji_id: Option<String>,
  // Standard emoji, this is the emoji itself (not a name, even though it says that)
  pub emoji_name: Option<String>,
  pub guild_id: Option<String>,
  // Can this be used?
  pub available: bool,
}

pub mod ipc;
pub mod ws;

pub use ws::{ChannelJoinPayload, MessageNotificationPayload, UpdatePayload, WsVoiceState};
