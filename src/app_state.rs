use std::{
  collections::HashMap,
  sync::{Arc, RwLock, atomic::AtomicBool},
};

use crate::{
  config::{Config, TransportMode},
  payloads::{MessageNotification, SoundboardSoundPayload},
  user::User,
  util::bridge::BridgeMessage,
};

/// Thread-safe shared state for background threads.
pub type SharedAppState = Arc<RwLock<AppState>>;

#[derive(Debug, Clone)]
pub struct AppState {
  pub config: Config,
  pub transport_mode: TransportMode,
  pub current_channel: String,
  pub is_open: bool,
  pub is_censor: bool, // Used in modded clients but not IPC
  pub voice_users: Vec<User>,
  pub messages: Vec<MessageNotification>,
  pub soundboard_cache: HashMap<String, Vec<SoundboardSoundPayload>>,

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
    Self {
      config: Config::default(),
      transport_mode: TransportMode::Ipc,
      current_channel: String::new(),
      is_open: false,
      is_censor: false,
      voice_users: vec![],
      messages: vec![],
      soundboard_cache: HashMap::new(),

      ws_sender: None,
      recording_keybind: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn send(&mut self, message: BridgeMessage) {
    if let Some(sender) = &self.ws_sender {
      sender.send(message).unwrap_or_default();
    }
  }

  pub fn notify(&mut self, notification: MessageNotification) {
    let messages_len = self.messages.len();

    // Keep the last 3 elements
    if messages_len > 3 {
      self.messages.drain(0..messages_len - 3);
    }

    self.messages.push(notification);
  }
}
