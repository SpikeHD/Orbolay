use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsVoiceState {
  pub user_id: String,
  pub username: Option<String>,
  pub avatar_url: Option<String>,
  pub channel_id: Option<String>,
  pub mute: Option<bool>,
  pub deaf: Option<bool>,
  pub speaking: Option<bool>,
  pub streaming: Option<bool>,
  pub camera: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ChannelJoinPayload {
  pub states: Vec<WsVoiceState>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UpdatePayload {
  pub state: WsVoiceState,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NotificationPayload {
  pub message: super::Notification,
}
