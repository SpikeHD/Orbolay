use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CornerAlignment {
  pub top: bool,
  pub left: bool,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  pub port: String,
  pub user_id: String,
  pub message_alignment: CornerAlignment,
  pub user_alignment: CornerAlignment,
  pub voice_semitransparent: bool,
  pub messages_semitransparent: bool,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      port: "6888".to_string(),
      user_id: String::new(),
      message_alignment: CornerAlignment {
        top: true,
        left: false,
      },
      user_alignment: CornerAlignment {
        top: true,
        left: true,
      },
      voice_semitransparent: true,
      messages_semitransparent: true,
    }
  }
}
