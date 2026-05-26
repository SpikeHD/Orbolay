use std::fmt::Display;

use freya::prelude::{Alignment, Gaps};
use serde::{Deserialize, Serialize};

#[cfg(not(target_os = "macos"))]
use crate::keys::bind::DEFAULT_OVERLAY_TOGGLE;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum AxisAlignment {
  Start,
  Center,
  End,
}

impl Display for AxisAlignment {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AxisAlignment::Start => write!(f, "start"),
      AxisAlignment::Center => write!(f, "center"),
      AxisAlignment::End => write!(f, "end"),
    }
  }
}

impl AxisAlignment {
  pub fn to_freya(&self) -> Alignment {
    match self {
      AxisAlignment::Start => Alignment::Start,
      AxisAlignment::Center => Alignment::Center,
      AxisAlignment::End => Alignment::End,
    }
  }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CornerAlignment {
  pub x: AxisAlignment,
  pub y: AxisAlignment,
}

impl CornerAlignment {
  pub fn from_str(s: impl AsRef<str>) -> Self {
    match s.as_ref().to_ascii_lowercase().as_str() {
      "topleft" => CornerAlignment {
        x: AxisAlignment::Start,
        y: AxisAlignment::Start,
      },
      "topright" => CornerAlignment {
        x: AxisAlignment::End,
        y: AxisAlignment::Start,
      },
      "bottomleft" => CornerAlignment {
        x: AxisAlignment::Start,
        y: AxisAlignment::End,
      },
      "bottomright" => CornerAlignment {
        x: AxisAlignment::End,
        y: AxisAlignment::End,
      },
      "topcenter" => CornerAlignment {
        x: AxisAlignment::Center,
        y: AxisAlignment::Start,
      },
      "bottomcenter" => CornerAlignment {
        x: AxisAlignment::Center,
        y: AxisAlignment::End,
      },
      "centerleft" => CornerAlignment {
        x: AxisAlignment::Start,
        y: AxisAlignment::Center,
      },
      "centerright" => CornerAlignment {
        x: AxisAlignment::End,
        y: AxisAlignment::Center,
      },
      _ => CornerAlignment {
        x: AxisAlignment::Start,
        y: AxisAlignment::Start,
      },
    }
  }

  pub fn to_gaps(&self, offset_x: i32, offset_y: i32) -> Gaps {
    let (top, bottom) = match self.y {
      AxisAlignment::Start => (offset_y as f32, 0.),
      AxisAlignment::End => (0., offset_y as f32),
      AxisAlignment::Center => (0., 0.),
    };
    let (left, right) = match self.x {
      AxisAlignment::Start => (offset_x as f32, 0.),
      AxisAlignment::End => (0., offset_x as f32),
      AxisAlignment::Center => (0., 0.),
    };
    Gaps::new(top, right, bottom, left)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  #[serde(default)]
  #[cfg(not(target_os = "macos"))]
  pub overlay_keybind: Option<Vec<String>>,
  #[serde(default)]
  pub port: Option<u16>,
  #[serde(default)]
  pub user_id: String,
  #[serde(default)]
  pub message_alignment: Option<String>,
  #[serde(default)]
  pub user_alignment: Option<String>,
  #[serde(default)]
  pub message_offset_x: i32,
  #[serde(default)]
  pub message_offset_y: i32,
  #[serde(default)]
  pub user_offset_x: i32,
  #[serde(default)]
  pub user_offset_y: i32,
  #[serde(default)]
  pub voice_semitransparent: Option<bool>,
  #[serde(default)]
  pub messages_semitransparent: bool,
  #[serde(default)]
  pub is_keybind_enabled: Option<bool>,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      #[cfg(not(target_os = "macos"))]
      overlay_keybind: Some(DEFAULT_OVERLAY_TOGGLE.clone()),
      port: Some(6888),
      user_id: String::new(),
      message_alignment: None,
      user_alignment: None,
      message_offset_x: 0,
      message_offset_y: 0,
      user_offset_x: 0,
      user_offset_y: 0,
      voice_semitransparent: None,
      messages_semitransparent: false,
      is_keybind_enabled: None,
    }
  }
}

pub fn config_dir() -> Option<std::path::PathBuf> {
  Some(dirs::config_dir()?.join("orbolay"))
}

pub fn save_config(config: &Config) {
  let Some(dir) = config_dir() else { return };
  if std::fs::create_dir_all(&dir).is_err() {
    return;
  }
  if let Ok(json) = serde_json::to_string_pretty(config) {
    std::fs::write(dir.join("config.json"), json).ok();
  }
}

pub fn load_config() -> Option<Config> {
  let dir = config_dir()?;
  let json = std::fs::read_to_string(dir.join("config.json")).ok()?;
  serde_json::from_str(&json).ok()
}

pub fn is_first_run() -> bool {
  load_config().is_none()
}
