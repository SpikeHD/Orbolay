use std::{cell::LazyCell, sync::atomic::{AtomicBool, Ordering}};

use rdev::Key;

use super::event::KeyEvent;

pub const DEFAULT_OVERLAY_TOGGLE: LazyCell<Vec<String>> = LazyCell::new(|| vec!["ControlLeft".into(), "BackQuote".into()]);
pub const DEFAULT_CONFIGURATOR_TOGGLE: LazyCell<Vec<String>> = LazyCell::new(|| vec!["ControlLeft".into(), "KeyP".into()]);

pub struct Keybind {
  pub keys: Vec<Key>,
  pub event: KeyEvent,
  active: AtomicBool,
}

impl Keybind {
  pub fn new(keys: Vec<Key>, event: KeyEvent) -> Self {
    Self {
      keys,
      event,
      active: AtomicBool::new(false),
    }
  }

  pub fn matches(&self, pressed: &[Key]) -> bool {
    pressed.len() == self.keys.len() && self.keys.iter().all(|k| pressed.contains(k))
  }

  pub fn active(&self) -> bool {
    self.active.load(Ordering::Relaxed)
  }

  pub fn set_active(&self, val: bool) {
    self.active.store(val, Ordering::Relaxed);
  }

  pub fn reset(&self) {
    self.active.store(false, Ordering::Relaxed);
  }
}

pub fn string_to_key(string: impl AsRef<str>) -> Option<Key> {
  serde_json::from_str::<Key>(string.as_ref()).ok()
}

pub fn strings_to_keys(strings: Vec<impl AsRef<str>>) -> Vec<Key> {
  strings
    .iter()
    .filter_map(|s| string_to_key(s))
    .collect()
}

pub fn key_to_string(key: &Key) -> String {
  serde_json::to_string(key).unwrap_or_default()
}

pub fn keys_to_strings(keys: Vec<Key>) -> Vec<String> {
  keys.iter().map(|k| key_to_string(k)).collect()
}

pub fn default_keybinds() -> Vec<Keybind> {
  vec![
    Keybind::new(
      strings_to_keys(DEFAULT_OVERLAY_TOGGLE.clone()),
      KeyEvent::ToggleOverlay,
    ),
    Keybind::new(strings_to_keys(DEFAULT_CONFIGURATOR_TOGGLE.clone()), KeyEvent::OpenConfigurator),
  ]
}
