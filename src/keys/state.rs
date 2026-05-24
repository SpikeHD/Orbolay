use std::sync::atomic::Ordering;
use std::time::Instant;

use rdev::{EventType, Key};

use super::{bind::Keybind, event::KeyEvent};

pub struct KeyState {
  pub pressed: Vec<Key>,
  pub last_update: Instant,
}

impl KeyState {
  pub fn new() -> Self {
    Self {
      pressed: Vec::new(),
      last_update: Instant::now(),
    }
  }

  pub fn press(&mut self, key: Key) {
    if !self.pressed.contains(&key) {
      self.pressed.push(key);
      self.last_update = Instant::now();
    }
  }

  pub fn release(&mut self, key: Key) {
    self.pressed.retain(|&k| k != key);
    self.last_update = Instant::now();
  }

  pub fn clear(&mut self) {
    self.pressed.clear();
    self.last_update = Instant::now();
  }
}

pub fn process(
  event_type: &EventType,
  key_state: &mut KeyState,
  keybinds: &[Keybind],
  tx: &std::sync::mpsc::Sender<KeyEvent>,
) {
  match event_type {
    EventType::KeyPress(k) => key_state.press(*k),
    EventType::KeyRelease(k) => key_state.release(*k),
    _ => return,
  }

  for bind in keybinds {
    let matches = bind.matches(&key_state.pressed);
    let was_active = bind.active.load(Ordering::Relaxed);

    if matches && !was_active {
      bind.active.store(true, Ordering::Relaxed);
      let _ = tx.send(bind.event.clone());
    } else if !matches && was_active {
      bind.active.store(false, Ordering::Relaxed);
    }
  }
}

pub fn reset(key_state: &mut KeyState, keybinds: &[Keybind]) {
  key_state.clear();
  for bind in keybinds {
    bind.reset();
  }
}
