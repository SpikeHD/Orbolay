use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CornerAlignment {
  pub top: bool,
  pub left: bool,
}

impl CornerAlignment {
  pub fn from_str(s: impl AsRef<str>) -> Self {
    match s.as_ref().to_ascii_lowercase().as_str() {
      "topleft" => CornerAlignment {
        top: true,
        left: true,
      },
      "topright" => CornerAlignment {
        top: true,
        left: false,
      },
      "bottomleft" => CornerAlignment {
        top: false,
        left: true,
      },
      "bottomright" => CornerAlignment {
        top: false,
        left: false,
      },
      _ => CornerAlignment {
        top: true,
        left: true,
      },
    }
  }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  pub port: Option<u16>,
  pub user_id: String,
  pub message_alignment: String,
  pub user_alignment: String,
  pub voice_semitransparent: bool,
  pub messages_semitransparent: bool,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      port: Some(6888),
      user_id: String::new(),
      message_alignment: "topright".into(),
      user_alignment: "topleft".into(),
      voice_semitransparent: true,
      messages_semitransparent: false,
    }
  }
}
