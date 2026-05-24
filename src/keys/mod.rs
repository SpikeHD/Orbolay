mod bind;
mod state;

pub mod event;

pub use event::KeyEvent;

use std::sync::{
  Arc, Mutex,
  atomic::{AtomicBool, Ordering},
  mpsc,
};
use std::thread;
use std::time::Duration;

use rdev::{Event, grab, listen};

use crate::{app_state::SharedAppState, log};

use bind::default_keybinds;
use state::{KeyState, process, reset};

pub fn watch_keybinds(shared: SharedAppState, keybind_tx: flume::Sender<KeyEvent>) {
  let keybinds = Arc::new(default_keybinds());
  let key_state = Arc::new(Mutex::new(KeyState::new()));
  let enabled = Arc::new(AtomicBool::new(true));
  let (tx, rx) = mpsc::channel::<KeyEvent>();

  // Clones for the monitoring thread
  let keybinds_monitor = keybinds.clone();
  let key_state_monitor = key_state.clone();
  let enabled_monitor = enabled.clone();

  thread::spawn(move || {
    let keybinds_grab = keybinds.clone();
    let key_state_grab = key_state.clone();
    let enabled_grab = enabled.clone();
    let tx_grab = tx.clone();

    let grab_cb = move |event: Event| {
      if enabled_grab.load(Ordering::Relaxed) {
        let mut state = key_state_grab.lock().unwrap();
        process(&event.event_type, &mut state, &keybinds_grab, &tx_grab);
      }
      Some(event)
    };

    if let Err(e) = grab(grab_cb) {
      log!("Failed to grab global hotkeys: {:?}", e);

      if let Err(e) = listen(move |event: Event| {
        if enabled.load(Ordering::Relaxed) {
          let mut state = key_state.lock().unwrap();
          process(&event.event_type, &mut state, &keybinds, &tx);
        }
      }) {
        log!("Failed to listen for global hotkeys: {:?}", e);
      }
    }
  });

  thread::spawn(move || {
    loop {
      let is_enabled = shared
        .read()
        .unwrap()
        .config
        .is_keybind_enabled
        .unwrap_or(true);
      enabled_monitor.store(is_enabled, Ordering::Relaxed);

      if !is_enabled {
        reset(&mut key_state_monitor.lock().unwrap(), &keybinds_monitor);
        thread::sleep(Duration::from_secs(1));
        continue;
      }

      while let Ok(event) = rx.try_recv() {
        log!("Key event: {:?}", event);
        let _ = keybind_tx.send(event);
      }

      {
        let mut state = key_state_monitor.lock().unwrap();
        if state.last_update.elapsed() > Duration::from_secs(5) {
          reset(&mut state, &keybinds_monitor);
        }
      }

      thread::sleep(Duration::from_millis(50));
    }
  });
}
