mod state;

pub mod bind;
pub mod convert;
pub mod event;

pub use event::KeyEvent;

use std::cell::RefCell;
use std::sync::{
  Arc, Mutex,
  atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use rdev::{Event, grab, listen};

use crate::{app_state::SharedAppState, log};

use bind::{DEFAULT_OVERLAY_TOGGLE, strings_to_keys};
use bind::{Keybind, default_keybinds};
use event::KeyEvent as KE;
use state::{KeyState, process};

pub fn watch_keybinds(shared: SharedAppState, keybind_tx: flume::Sender<KeyEvent>) {
  let enabled = Arc::new(AtomicBool::new(true));
  let keybinds: Arc<Mutex<Vec<Keybind>>> = Arc::new(Mutex::new(default_keybinds()));
  let recording = shared.read().unwrap().recording_keybind.clone();

  // Poll config every second to sync keybind/enabled state
  {
    let enabled = enabled.clone();
    let keybinds = keybinds.clone();
    let shared = shared.clone();
    thread::spawn(move || {
      loop {
        let config = shared.read().unwrap().config.clone();

        enabled.store(config.is_keybind_enabled.unwrap_or(true), Ordering::Relaxed);

        let overlay_keys = strings_to_keys(
          config
            .overlay_keybind
            .clone()
            .unwrap_or_else(|| DEFAULT_OVERLAY_TOGGLE.clone()),
        );

        let mut kbs = keybinds.lock().unwrap();
        for kb in kbs.iter_mut() {
          if matches!(kb.event, KE::ToggleOverlay) && kb.keys != overlay_keys {
            kb.keys = overlay_keys.clone();
            kb.reset();
            break;
          }
        }

        thread::sleep(Duration::from_secs(1));
      }
    });
  }

  // Key listener thread
  thread::spawn(move || {
    let keybind_tx_grab = keybind_tx.clone();
    let enabled_grab = enabled.clone();
    let keybinds_grab = keybinds.clone();
    let recording_grab = recording.clone();
    let key_state_grab = RefCell::new(KeyState::new());

    let grab_result = grab(move |event: Event| {
      if !recording_grab.load(Ordering::Relaxed) && enabled_grab.load(Ordering::Relaxed) {
        let kbs = keybinds_grab.lock().unwrap();
        process(
          &event.event_type,
          &mut key_state_grab.borrow_mut(),
          &kbs,
          &keybind_tx_grab,
        );
      }
      Some(event)
    });

    if let Err(e) = grab_result {
      log!("Failed to grab global hotkeys: {:?}", e);

      let key_state_listen = RefCell::new(KeyState::new());
      if let Err(e) = listen(move |event: Event| {
        if !recording.load(Ordering::Relaxed) && enabled.load(Ordering::Relaxed) {
          let kbs = keybinds.lock().unwrap();
          process(
            &event.event_type,
            &mut key_state_listen.borrow_mut(),
            &kbs,
            &keybind_tx,
          );
        }
      }) {
        log!("Failed to listen for global hotkeys: {:?}", e);
      }
    }
  });
}
