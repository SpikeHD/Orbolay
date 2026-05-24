mod bind;
mod state;

pub mod event;

pub use event::KeyEvent;

use std::cell::RefCell;
use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

use rdev::{Event, grab, listen};

use crate::{app_state::SharedAppState, log};

use bind::default_keybinds;
use state::{KeyState, process};

pub fn watch_keybinds(shared: SharedAppState, keybind_tx: flume::Sender<KeyEvent>) {
  let enabled = Arc::new(AtomicBool::new(true));
  let enabled_monitor = enabled.clone();

  thread::spawn(move || loop {
    let is_enabled = shared.read().unwrap().config.is_keybind_enabled.unwrap_or(true);
    enabled_monitor.store(is_enabled, Ordering::Relaxed);
    thread::sleep(Duration::from_secs(1));
  });

  thread::spawn(move || {
    let keybind_tx_grab = keybind_tx.clone();
    let enabled_grab = enabled.clone();
    let key_state_grab = RefCell::new(KeyState::new());
    let keybinds_grab = default_keybinds();

    let grab_result = grab(move |event: Event| {
      if enabled_grab.load(Ordering::Relaxed) {
        process(&event.event_type, &mut key_state_grab.borrow_mut(), &keybinds_grab, &keybind_tx_grab);
      }
      Some(event)
    });

    if let Err(e) = grab_result {
      log!("Failed to grab global hotkeys: {:?}", e);

      let key_state_listen = RefCell::new(KeyState::new());
      let keybinds_listen = default_keybinds();

      if let Err(e) = listen(move |event: Event| {
        if enabled.load(Ordering::Relaxed) {
          process(&event.event_type, &mut key_state_listen.borrow_mut(), &keybinds_listen, &keybind_tx);
        }
      }) {
        log!("Failed to listen for global hotkeys: {:?}", e);
      }
    }
  });
}
