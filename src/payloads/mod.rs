use serde::Deserialize;
use std::sync::Arc;

pub type NotificationFn = Arc<dyn Fn() + Send + Sync>;

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationKind {
  Primary,
  Secondary,
}

#[derive(Clone)]
pub struct NotificationAction {
  pub label: String,
  pub action: NotificationFn,
  pub kind: NotificationKind,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
  pub title: String,
  pub body: String,
  pub icon: String,
  pub guild_id: Option<String>,
  pub channel_id: Option<String>,
  pub message_id: Option<String>,
  #[serde(default, skip_deserializing)]
  pub timestamp: Option<i64>,
  #[serde(skip)]
  pub actions: Option<Vec<NotificationAction>>,
}

impl PartialEq for Notification {
  fn eq(&self, other: &Self) -> bool {
    self.title == other.title
      && self.body == other.body
      && self.icon == other.icon
      && self.guild_id == other.guild_id
      && self.channel_id == other.channel_id
      && self.message_id == other.message_id
      && self.timestamp == other.timestamp
    // Actions are not compared as they are functions
  }
}

impl std::fmt::Debug for Notification {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Notification")
      .field("title", &self.title)
      .field("body", &self.body)
      .field("icon", &self.icon)
      .field("guild_id", &self.guild_id)
      .field("channel_id", &self.channel_id)
      .field("message_id", &self.message_id)
      .field("timestamp", &self.timestamp)
      .field("actions", &self.actions.is_some())
      .finish()
  }
}

impl Default for Notification {
  fn default() -> Self {
    Self {
      title: String::new(),
      body: String::new(),
      icon: String::new(),
      guild_id: None,
      channel_id: None,
      message_id: None,
      timestamp: Some(chrono::Utc::now().timestamp()),
      actions: None,
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

pub use ws::{ChannelJoinPayload, NotificationPayload, UpdatePayload, WsVoiceState};
