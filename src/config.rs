use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum Alignment {
  Start,
  Center,
  End,
}

impl ToString for Alignment {
  fn to_string(&self) -> String {
    match self {
      Alignment::Start => "start".into(),
      Alignment::Center => "center".into(),
      Alignment::End => "end".into(),
    }
  }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CornerAlignment {
  pub x: Alignment,
  pub y: Alignment,
}

impl CornerAlignment {
  pub fn from_str(s: impl AsRef<str>) -> Self {
    match s.as_ref().to_ascii_lowercase().as_str() {
      "topleft" => CornerAlignment {
        x: Alignment::Start,
        y: Alignment::Start,
      },
      "topright" => CornerAlignment {
        x: Alignment::End,
        y: Alignment::Start,
      },
      "bottomleft" => CornerAlignment {
        x: Alignment::Start,
        y: Alignment::End,
      },
      "bottomright" => CornerAlignment {
        x: Alignment::End,
        y: Alignment::End,
      },
      "topcenter" => CornerAlignment {
        x: Alignment::Center,
        y: Alignment::Start,
      },
      "bottomcenter" => CornerAlignment {
        x: Alignment::Center,
        y: Alignment::End,
      },
      "centerleft" => CornerAlignment {
        x: Alignment::Start,
        y: Alignment::Center,
      },
      "centerright" => CornerAlignment {
        x: Alignment::End,
        y: Alignment::Center,
      },
      _ => CornerAlignment {
        x: Alignment::Start,
        y: Alignment::Start,
      }
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
  pub is_keybind_enabled: bool,
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
      is_keybind_enabled: true,
    }
  }
}
