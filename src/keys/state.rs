use crate::log;
use std::time::{Duration, Instant};

use rdev::{EventType, Key};

use super::{bind::Keybind, event::KeyEvent};

const STALE_TIMEOUT: Duration = Duration::from_secs(5);

pub struct KeyState {
  pub pressed: Vec<Key>,
  last_update: Instant,
}

impl KeyState {
  pub fn new() -> Self {
    Self {
      pressed: Vec::new(),
      last_update: Instant::now(),
    }
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
  tx: &flume::Sender<KeyEvent>,
) {
  if key_state.last_update.elapsed() > STALE_TIMEOUT {
    key_state.clear();
    for bind in keybinds {
      bind.reset();
    }
  }

  match event_type {
    EventType::KeyPress(k) => key_state.press(*k),
    EventType::KeyRelease(k) => key_state.release(*k),
    _ => return,
  }

  for bind in keybinds {
    let matches = bind.matches(&key_state.pressed);
    let was_active = bind.active();

    if matches && !was_active {
      bind.set_active(true);
      log!("Key event: {:?}", bind.event);
      let _ = tx.send(bind.event.clone());
    } else if !matches && was_active {
      bind.set_active(false);
    }
  }
}
