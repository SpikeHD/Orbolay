use std::sync::mpsc;

use notify::{Event, Result, Watcher};

use crate::{
  app_state::SharedAppState,
  config::{config_dir, load_config},
  log,
};

pub fn start_config_watcher(shared: SharedAppState, redraw_tx: flume::Sender<()>) {
  log!("Starting config file notification thread");
  std::thread::spawn(move || {
    let (tx, rx) = mpsc::channel::<Result<Event>>();

    let mut watcher = notify::recommended_watcher(tx).expect("Failed to create config watcher");

    let config_path = match config_dir() {
      Some(path) => path.join("config.json"),
      None => {
        eprintln!("Failed to get config path");
        return;
      }
    };

    watcher
      .watch(&config_path, notify::RecursiveMode::NonRecursive)
      .expect("Failed to watch config file");

    loop {
      match rx.recv() {
        Ok(Ok(event)) => {
          if !event.kind.is_modify() {
            continue;
          }

          if event.paths.iter().any(|p| p == &config_path) {
            println!("Config file changed, reloading...");
            if let Some(new_config) = load_config() {
              let mut state = shared.write().unwrap();
              state.config = new_config;
              redraw_tx.send(()).ok();
              log!("Config reloaded successfully");
            } else {
              eprintln!("Failed to reload config file");
            }
          }
        }
        Ok(Err(e)) => eprintln!("Watcher error: {}", e),
        Err(e) => eprintln!("Watcher channel error: {}", e),
      }
    }
  });
}
