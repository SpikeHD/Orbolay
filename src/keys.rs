use std::sync::{atomic::{AtomicBool, Ordering}, Arc};

use device_query::{DeviceQuery, DeviceState, Keycode};
use freya::prelude::*;

use crate::{app_state::AppState, log};

// TODO configurable
static KEYBIND: [Keycode; 2] = [Keycode::LControl, Keycode::Grave];

pub fn watch_keybinds(mut app_state: Signal<AppState, SyncStorage>) {
  std::thread::spawn(move || {
    let pressed = AtomicBool::new(false);

    loop {
      std::thread::sleep(std::time::Duration::from_millis(50));

      let state = DeviceState::new();
      let keys = state.get_keys();

      // Check if only the keybind keys are being pressed
      if keys.len() == KEYBIND.len() {
        let mut all_match = true;

        for (i, key) in KEYBIND.iter().enumerate() {
          if keys[i] != *key {
            all_match = false;
            break;
          }
        }

        if all_match && !pressed.load(Ordering::Relaxed) {
          (*app_state.write()).is_open = !app_state().is_open;
          pressed.store(true, Ordering::Relaxed);
          log!("Opening overlay");
        } else if pressed.load(Ordering::Relaxed) {
          pressed.store(false, Ordering::Relaxed);
          log!("Closing overlay");
        }
      }
    }
  });
}