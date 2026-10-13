use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use rdev::{EventType, Key};

use crate::KeyEvent;
use orbolay_logging::log;

const STALE_TIMEOUT: Duration = Duration::from_secs(5);

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

#[non_exhaustive]
pub struct KeyState {
  pub pressed: Vec<Key>,
  last_update: Instant,
}

impl Default for KeyState {
  fn default() -> Self {
    Self {
      pressed: Vec::new(),
      last_update: Instant::now(),
    }
  }
}

impl KeyState {
  pub fn new() -> Self {
    Self::default()
  }

  fn press(&mut self, key: Key) {
    if !self.pressed.contains(&key) {
      self.pressed.push(key);
      self.last_update = Instant::now();
    }
  }

  fn release(&mut self, key: Key) {
    self.pressed.retain(|&k| k != key);
    self.last_update = Instant::now();
  }

  fn clear(&mut self) {
    self.pressed.clear();
    self.last_update = Instant::now();
  }
}

pub fn process(
  event_type: &EventType,
  key_state: &mut KeyState,
  keybinds: &[Keybind],
) -> Vec<KeyEvent> {
  if key_state.last_update.elapsed() > STALE_TIMEOUT {
    key_state.clear();
    for bind in keybinds {
      bind.reset();
    }
  }

  match event_type {
    EventType::KeyPress(k) => key_state.press(*k),
    EventType::KeyRelease(k) => key_state.release(*k),
    _ => return vec![],
  }

  let mut fired = vec![];

  for bind in keybinds {
    let matches = bind.matches(&key_state.pressed);
    let was_active = bind.active();

    if matches && !was_active {
      bind.set_active(true);
      log!("Key event: {:?}", bind.event);
      fired.push(bind.event.clone());
    } else if !matches && was_active {
      bind.set_active(false);
    }
  }

  fired
}
