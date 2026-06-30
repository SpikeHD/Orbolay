use std::{
  collections::HashMap,
  sync::{Arc, RwLock, atomic::AtomicBool},
};

use crate::{
  config::{Config, TransportMode},
  payloads::{Notification, SoundboardSoundPayload},
  user::{PremiumType, User},
  util::bridge::BridgeMessage,
};

/// Thread-safe shared state for background threads.
pub type SharedAppState = Arc<RwLock<AppState>>;

#[derive(Clone)]
pub struct AppHandle {
  shared: SharedAppState,
  redraw_tx: flume::Sender<()>,
}

impl AppHandle {
  pub fn new(shared: SharedAppState, redraw_tx: flume::Sender<()>) -> Self {
    Self { shared, redraw_tx }
  }

  pub fn shared(&self) -> &SharedAppState {
    &self.shared
  }

  pub fn read<T>(&self, f: impl FnOnce(&AppState) -> T) -> T {
    let state = self.shared.read().unwrap();
    f(&state)
  }

  pub fn update<T>(&self, f: impl FnOnce(&mut AppState) -> T) -> T {
    let result = {
      let mut state = self.shared.write().unwrap();
      f(&mut state)
    };
    let _ = self.redraw_tx.send(());
    result
  }

  pub fn update_if_changed(&self, f: impl FnOnce(&mut AppState) -> bool) {
    let changed = {
      let mut state = self.shared.write().unwrap();
      f(&mut state)
    };

    if changed {
      let _ = self.redraw_tx.send(());
    }
  }

  pub fn send(&self, message: BridgeMessage) {
    self.read(|state| state.send(message));
  }

  pub fn notify(&self, notification: Notification) {
    self.update(|state| state.notify(notification));
  }

  pub fn redraw(&self) {
    let _ = self.redraw_tx.send(());
  }
}

#[derive(Debug, Clone)]
pub struct AppState {
  pub config: Config,
  pub user_id: String,
  pub premium_type: PremiumType,
  pub transport_mode: TransportMode,
  pub current_channel: String,
  pub current_guild_id: String,
  pub is_open: bool,
  pub is_censor: bool, // Used in modded clients but not IPC
  pub voice_users: Vec<User>,
  pub messages: Vec<Notification>,
  pub soundboard_cache: HashMap<String, Vec<SoundboardSoundPayload>>,

  // Name caches
  pub guild_names: HashMap<String, String>,
  pub channel_names: HashMap<String, String>,

  pub ws_sender: Option<flume::Sender<BridgeMessage>>,

  pub recording_keybind: Arc<AtomicBool>,
}

impl Default for AppState {
  fn default() -> Self {
    Self::new()
  }
}

impl AppState {
  pub fn new() -> Self {
    let mut default_guild_names: HashMap<String, String> = HashMap::new();
    default_guild_names.insert("0".into(), "Default".into());

    Self {
      config: Config::default(),
      user_id: String::new(),
      premium_type: PremiumType::None,
      transport_mode: TransportMode::Ipc,
      current_channel: String::new(),
      current_guild_id: String::new(),
      is_open: false,
      is_censor: false,
      voice_users: vec![],
      messages: vec![],
      soundboard_cache: HashMap::new(),

      guild_names: default_guild_names,
      channel_names: HashMap::new(),

      ws_sender: None,
      recording_keybind: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn send(&self, message: BridgeMessage) {
    if let Some(sender) = &self.ws_sender {
      sender.send(message).unwrap_or_default();
    }
  }

  pub fn notify(&mut self, notification: Notification) {
    let messages_len = self.messages.len();

    // Keep the last 3 elements
    if messages_len > 3 {
      self.messages.drain(0..messages_len - 3);
    }

    self.messages.push(notification);
  }
}
