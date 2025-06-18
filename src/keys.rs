use std::sync::atomic::{AtomicBool, Ordering};

use device_query::{DeviceQuery, DeviceState, Keycode};
use freya::prelude::*;

use crate::{app_state::AppState, log};

// TODO configurable
static KEYBIND: [Keycode; 2] = [Keycode::LControl, Keycode::Grave];

pub fn watch_keybinds(mut app_state: Signal<AppState, SyncStorage>, platform: PlatformSender) {
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
          app_state.write().is_open = !app_state().is_open;
          pressed.store(true, Ordering::Relaxed);
          platform.with_window(move |w| {
            let _ = w.set_cursor_hittest(app_state.read().is_open);
          });
          log!("Toggling overlay");
        } else if !all_match && pressed.load(Ordering::Relaxed) {
          pressed.store(false, Ordering::Relaxed);
        }
      } else {
        pressed.store(false, Ordering::Relaxed);
      }
    }
  });
}
