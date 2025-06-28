use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CornerAlignment {
  pub top: bool,
  pub left: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  pub port: u16,
  pub user_id: String,
  pub message_alignment: String,
  pub user_alignment: String,
  pub voice_semitransparent: bool,
  pub messages_semitransparent: bool,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      port: 6888,
      user_id: String::new(),
      message_alignment: "topright".into(),
      user_alignment: "topleft".into(),
      voice_semitransparent: true,
      messages_semitransparent: true,
    }
  }
}
