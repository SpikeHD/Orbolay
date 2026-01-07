use std::sync::{
  Arc, Mutex,
  atomic::{AtomicBool, Ordering},
  mpsc,
};
use std::thread;
use std::time::{Duration, Instant};

use freya::prelude::*;
use rdev::{Event, EventType, Key, grab, listen};

use crate::{app_state::AppState, log};

// TODO configurable
static KEYBIND: [Key; 2] = [Key::ControlLeft, Key::BackQuote];

#[derive(Debug, Clone)]
struct KeyState {
  pressed_keys: Vec<Key>,
  last_update: Instant,
}

impl KeyState {
  fn new() -> Self {
    Self {
      pressed_keys: Vec::new(),
      last_update: Instant::now(),
    }
  }

  fn add_key(&mut self, key: Key) {
    if !self.pressed_keys.contains(&key) {
      self.pressed_keys.push(key);
      self.last_update = Instant::now();
    }
  }

  fn remove_key(&mut self, key: Key) {
    self.pressed_keys.retain(|&k| k != key);
    self.last_update = Instant::now();
  }

  fn is_keybind_pressed(&self) -> bool {
    if self.pressed_keys.len() != KEYBIND.len() {
      return false;
    }

    let mut keybind_sorted = KEYBIND.to_vec();
    let mut pressed_sorted = self.pressed_keys.clone();

    keybind_sorted.sort_by_key(|k| format!("{:?}", k));
    pressed_sorted.sort_by_key(|k| format!("{:?}", k));

    keybind_sorted == pressed_sorted
  }

  fn clear(&mut self) {
    self.pressed_keys.clear();
    self.last_update = Instant::now();
  }
}

pub fn watch_keybinds(mut app_state: Signal<AppState, SyncStorage>, platform: PlatformSender) {
  let key_state = Arc::new(Mutex::new(KeyState::new()));
  let pressed = Arc::new(AtomicBool::new(false));
  let enabled = Arc::new(AtomicBool::new(true));

  let (tx, rx) = mpsc::channel();

  // Event handler states
  let key_state_handler = key_state.clone();
  let pressed_handler = pressed.clone();
  let enabled_handler = enabled.clone();
  let tx_handler = tx.clone();

  // Monitoring thread states
  let key_state_monitor = key_state.clone();
  let pressed_monitor = pressed.clone();
  let enabled_monitor = enabled.clone();

  thread::spawn(move || {
    let callback = move |event: Event| {
      if !enabled_handler.load(Ordering::Relaxed) {
        return Some(event);
      }

      match event.event_type {
        EventType::KeyPress(key) => {
          let mut state = key_state_handler.lock().unwrap();
          state.add_key(key);

          if state.is_keybind_pressed() && !pressed_handler.load(Ordering::Relaxed) {
            pressed_handler.store(true, Ordering::Relaxed);
            let _ = tx_handler.send(true);
          }
        }
        EventType::KeyRelease(key) => {
          let mut state = key_state_handler.lock().unwrap();
          state.remove_key(key);

          if !state.is_keybind_pressed() && pressed_handler.load(Ordering::Relaxed) {
            pressed_handler.store(false, Ordering::Relaxed);
          }
        }
        _ => {}
      }

      // Always return the event, we should never block it
      Some(event)
    };

    // This blocks
    if let Err(e) = grab(callback) {
      log!("Failed to grab global hotkeys: {:?}", e);

      // Clone for fallback listen mode
      let key_state_fallback = key_state.clone();
      let pressed_fallback = pressed.clone();
      let enabled_fallback = enabled.clone();
      let tx_fallback = tx.clone();

      // If grab fails, we can at least try listening mode (doesn't work on Wayland though)
      if let Err(e) = listen(move |event: Event| {
        if !enabled_fallback.load(Ordering::Relaxed) {
          return;
        }

        match event.event_type {
          EventType::KeyPress(key) => {
            let mut state = key_state_fallback.lock().unwrap();
            state.add_key(key);

            if state.is_keybind_pressed() && !pressed_fallback.load(Ordering::Relaxed) {
              pressed_fallback.store(true, Ordering::Relaxed);
              let _ = tx_fallback.send(true);
            }
          }
          EventType::KeyRelease(key) => {
            let mut state = key_state_fallback.lock().unwrap();
            state.remove_key(key);

            if !state.is_keybind_pressed() && pressed_fallback.load(Ordering::Relaxed) {
              pressed_fallback.store(false, Ordering::Relaxed);
            }
          }
          _ => {}
        }
      }) {
        log!("Failed to listen for global hotkeys: {:?}", e);
      }
    }
  });

  // Thread for keeping track of keybind state and toggling overlay
  thread::spawn(move || {
    loop {
      let is_enabled = app_state.read().config.is_keybind_enabled;
      enabled_monitor.store(is_enabled, Ordering::Relaxed);

      if !is_enabled {
        // Clear key state when disabled
        key_state_monitor.lock().unwrap().clear();
        pressed_monitor.store(false, Ordering::Relaxed);
        thread::sleep(Duration::from_secs(1));
        continue;
      }

      if rx.try_recv().is_ok() {
        log!("Toggling overlay");
        app_state.write().is_open = !app_state().is_open;

        platform.with_window(move |w| {
          let _ = w.set_cursor_hittest(app_state.read().is_open);
        });
      }

      // Clear old key states (in case we missed a key release)
      {
        let mut state = key_state_monitor.lock().unwrap();
        if state.last_update.elapsed() > Duration::from_secs(5) {
          state.clear();
          pressed_monitor.store(false, Ordering::Relaxed);
        }
      }

      thread::sleep(Duration::from_millis(50));
    }
  });
}
