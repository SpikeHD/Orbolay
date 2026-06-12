use serde::Deserialize;

use crate::user::PremiumType;

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
pub struct RpcVoiceState {
  pub nick: Option<String>,
  pub mute: bool,
  pub volume: f32,
  pub voice_state: VoiceStateState,
  pub user: VoiceStateUser,
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
pub struct NotificationInner {
  pub id: Option<String>,
  pub guild_id: Option<String>,
  pub channel_id: Option<String>,
  #[serde(rename = "type")]
  pub message_type: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct GetUserPayload {
  pub id: String,
  #[serde(default, deserialize_with = "deserialize_premium_type")]
  pub premium_type: PremiumType,
}

fn deserialize_premium_type<'de, D>(deserializer: D) -> Result<PremiumType, D::Error>
where
  D: serde::Deserializer<'de>,
{
  let value = u64::deserialize(deserializer)?;
  PremiumType::try_from(value).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NotificationCreatePayload {
  pub channel_id: Option<String>,
  pub message: Option<NotificationInner>,
  pub icon_url: Option<String>,
  pub title: String,
  pub body: String,
}
