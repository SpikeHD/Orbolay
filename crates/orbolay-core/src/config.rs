use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum TransportMode {
  #[default]
  Ipc,
  Websocket,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum DisplayVoiceMembers {
  Always,
  #[default]
  AlwaysSemiTransparent,
  WhenSpeaking,
}

impl Display for TransportMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TransportMode::Ipc => write!(f, "ipc"),
      TransportMode::Websocket => write!(f, "websocket"),
    }
  }
}

impl From<String> for TransportMode {
  fn from(value: String) -> Self {
    match value.as_ref() {
      "ipc" => TransportMode::Ipc,
      "websocket" => TransportMode::Websocket,
      _ => TransportMode::Ipc,
    }
  }
}

impl Display for DisplayVoiceMembers {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      DisplayVoiceMembers::Always => write!(f, "always"),
      DisplayVoiceMembers::AlwaysSemiTransparent => write!(f, "always (semi-transparent)"),
      DisplayVoiceMembers::WhenSpeaking => write!(f, "only when speaking"),
    }
  }
}

impl From<String> for DisplayVoiceMembers {
  fn from(value: String) -> Self {
    match value.as_ref() {
      "always" => DisplayVoiceMembers::Always,
      "always (semi-transparent)" => DisplayVoiceMembers::AlwaysSemiTransparent,
      "only when speaking" => DisplayVoiceMembers::WhenSpeaking,
      _ => DisplayVoiceMembers::Always,
    }
  }
}

fn default_overlay_keybind() -> Option<Vec<String>> {
  Some(vec!["ControlLeft".into(), "BackQuote".into()])
}

fn default_accent() -> (u8, u8, u8) {
  (30, 31, 35)
}

fn default_text() -> (u8, u8, u8) {
  (255, 255, 255)
}

fn default_border_radius() -> f32 {
  10.
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  #[serde(default = "default_overlay_keybind")]
  #[cfg(not(target_os = "macos"))]
  pub overlay_keybind: Option<Vec<String>>,
  #[serde(default)]
  pub display_idx: Option<usize>,
  #[serde(default)]
  pub port: Option<u16>,
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
  pub display_voice_members: Option<DisplayVoiceMembers>,
  #[serde(default)]
  pub messages_semitransparent: bool,
  #[serde(default)]
  pub is_keybind_enabled: Option<bool>,
  #[serde(default)]
  pub transport_mode: TransportMode,
  #[serde(default)]
  pub software_rendering: Option<bool>,
  #[serde(default = "default_accent")]
  pub accent: (u8, u8, u8),
  #[serde(default = "default_text")]
  pub text_color: (u8, u8, u8),
  #[serde(default = "default_border_radius")]
  pub border_radius: f32,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      #[cfg(not(target_os = "macos"))]
      overlay_keybind: default_overlay_keybind(),
      display_idx: None,
      port: Some(6888),
      message_alignment: None,
      user_alignment: None,
      message_offset_x: 0,
      message_offset_y: 0,
      user_offset_x: 0,
      user_offset_y: 0,
      display_voice_members: Some(DisplayVoiceMembers::AlwaysSemiTransparent),
      messages_semitransparent: false,
      is_keybind_enabled: None,
      transport_mode: TransportMode::Ipc,
      software_rendering: None,
      accent: default_accent(),
      text_color: default_text(),
      border_radius: 10.,
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
