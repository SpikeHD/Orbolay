use std::sync::mpsc;

use notify::{Event, Result, Watcher};

use crate::{
  app_state::SharedAppState,
  config::{config_dir, load_config},
  error, log, warn,
};

pub fn start_config_watcher(shared: SharedAppState, redraw_tx: flume::Sender<()>) {
  log!("Starting config file notification thread");
  std::thread::spawn(move || {
    let (tx, rx) = mpsc::channel::<Result<Event>>();

    let mut watcher = match notify::recommended_watcher(tx) {
      Ok(w) => w,
      Err(e) => {
        warn!(
          "Failed to create config watcher: {}. Live config reload disabled.",
          e
        );
        return;
      }
    };

    let config_path = match config_dir() {
      Some(path) => path.join("config.json"),
      None => {
        warn!("Failed to get config path; live config reload disabled");
        return;
      }
    };

    if let Err(e) = watcher.watch(&config_path, notify::RecursiveMode::NonRecursive) {
      warn!(
        "Failed to watch config file at {:?}: {}. Live config reload disabled.",
        config_path, e
      );
      return;
    }

    loop {
      match rx.recv() {
        Ok(Ok(event)) => {
          if !event.kind.is_modify() {
            continue;
          }

          if event.paths.iter().any(|p| p == &config_path) {
            log!("Config file changed, reloading...");
            if let Some(new_config) = load_config() {
              let mut state = shared.write().unwrap();
              state.config = new_config;
              redraw_tx.send(()).ok();
              log!("Config reloaded successfully");
            } else {
              warn!("Failed to reload config file");
            }
          }
        }
        Ok(Err(e)) => error!("Watcher error: {}", e),
        Err(e) => error!("Watcher channel error: {}", e),
      }
    }
  });
}
