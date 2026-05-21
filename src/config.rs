use std::fmt::Display;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum Alignment {
  Start,
  Center,
  End,
}

impl Display for Alignment {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Alignment::Start => write!(f, "start"),
      Alignment::Center => write!(f, "center"),
      Alignment::End => write!(f, "end"),
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
      },
    }
  }

  pub fn padding(&self, offset_x: i32, offset_y: i32) -> String {
    let (top, bottom) = match self.y {
      Alignment::Start => (offset_y, 0),
      Alignment::End => (0, offset_y),
      Alignment::Center => (0, 0),
    };
    let (left, right) = match self.x {
      Alignment::Start => (offset_x, 0),
      Alignment::End => (0, offset_x),
      Alignment::Center => (0, 0),
    };
    format!("{top} {right} {bottom} {left}")
  }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  pub port: Option<u16>,
  pub user_id: String,
  pub message_alignment: String,
  pub user_alignment: String,
  #[serde(default)]
  pub message_offset_x: i32,
  #[serde(default)]
  pub message_offset_y: i32,
  #[serde(default)]
  pub user_offset_x: i32,
  #[serde(default)]
  pub user_offset_y: i32,
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
      message_offset_x: 0,
      message_offset_y: 0,
      user_offset_x: 0,
      user_offset_y: 0,
      voice_semitransparent: true,
      messages_semitransparent: false,
      is_keybind_enabled: true,
    }
  }
}
