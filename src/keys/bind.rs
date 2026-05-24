use std::sync::atomic::{AtomicBool, Ordering};

use rdev::Key;

use super::event::KeyEvent;

pub struct Keybind {
  pub keys: Vec<Key>,
  pub event: KeyEvent,
  pub active: AtomicBool,
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

  pub fn reset(&self) {
    self.active.store(false, Ordering::Relaxed);
  }
}

pub fn default_keybinds() -> Vec<Keybind> {
  vec![
    Keybind::new(
      vec![Key::ControlLeft, Key::BackQuote],
      KeyEvent::ToggleOverlay,
    ),
    Keybind::new(vec![Key::KeyC], KeyEvent::OpenConfigurator),
  ]
}
