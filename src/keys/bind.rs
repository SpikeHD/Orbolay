use std::sync::atomic::{AtomicBool, Ordering};

use orbolay_keys::{DEFAULT_OVERLAY_TOGGLE, strings_to_keys};
use rdev::Key;

use super::event::KeyEvent;

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



pub fn default_keybinds() -> Vec<Keybind> {
  vec![
    Keybind::new(
      strings_to_keys(DEFAULT_OVERLAY_TOGGLE.clone()),
      KeyEvent::ToggleOverlay,
    ),
    Keybind::new(vec![Key::KeyC], KeyEvent::OpenConfigurator),
  ]
}
