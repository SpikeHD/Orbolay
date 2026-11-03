use std::{
  collections::{HashMap, VecDeque},
  sync::{Arc, RwLock, atomic::AtomicBool},
  time::{Duration, Instant},
};

use crate::{
  config::{Config, TransportMode},
  payloads::{Notification, SoundboardSoundPayload},
  user::{PremiumType, User, UserVoiceState},
  util::bridge::BridgeMessage,
};

const MAX_PENDING_CHAT: usize = 100;

/// Thread-safe shared state for background threads.
pub type SharedAppState = Arc<RwLock<AppState>>;

#[derive(Clone)]
pub struct AppHandle {
  shared: SharedAppState,
  redraw_tx: flume::Sender<()>,
  open_config_tx: flume::Sender<()>,
  grace_tx: flume::Sender<(String, Instant)>,
}

impl AppHandle {
  pub fn new(
    shared: SharedAppState,
    redraw_tx: flume::Sender<()>,
    open_config_tx: flume::Sender<()>,
  ) -> Self {
    let (grace_tx, grace_rx) = flume::unbounded::<(String, Instant)>();
    let app = Self {
      shared,
      redraw_tx,
      open_config_tx,
      grace_tx,
    };

    let grace_app = app.clone();
    std::thread::Builder::new()
      .name("orbolay-grace".into())
      .spawn(move || {
        for (user_id, until) in grace_rx.iter() {
          if let Some(wait) = until.checked_duration_since(Instant::now()) {
            std::thread::sleep(wait);
          }
          grace_app.end_grace(&user_id, until);
        }
      })
      .expect("failed to spawn the grace thread");

    app
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

  pub fn open_config(&self) -> Result<(), flume::SendError<()>> {
    self.open_config_tx.send(())
  }

  pub fn start_grace(&self, user: &mut User, grace_ms: u32) {
    if grace_ms == 0 {
      user.voice_state = UserVoiceState::NotSpeaking;
      return;
    }
    if user.grace_until.is_some() {
      return;
    }

    let until = Instant::now() + Duration::from_millis(u64::from(grace_ms));
    user.grace_until = Some(until);
    let _ = self.grace_tx.send((user.id.clone(), until));
  }

  fn end_grace(&self, user_id: &str, until: Instant) {
    self.update_if_changed(|state| {
      let Some(user) = state
        .voice_users
        .iter_mut()
        .find(|user| user.id == user_id && user.grace_until == Some(until))
      else {
        return false;
      };
      user.grace_until = None;
      if user.voice_state == UserVoiceState::Speaking {
        user.voice_state = UserVoiceState::NotSpeaking;
      }
      true
    });
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
  pub chat_messages: Vec<Notification>,
  pub pending_messages: VecDeque<Notification>,
  pub pending_chat: VecDeque<Notification>,
  pub soundboard_cache: HashMap<String, Vec<SoundboardSoundPayload>>,

  // Name caches
  pub guild_names: HashMap<String, String>,
  pub channel_names: HashMap<String, String>,

  pub transport_sender: Option<flume::Sender<BridgeMessage>>,

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
      chat_messages: vec![],
      pending_messages: VecDeque::new(),
      pending_chat: VecDeque::new(),
      soundboard_cache: HashMap::new(),

      guild_names: default_guild_names,
      channel_names: HashMap::new(),

      transport_sender: None,
      recording_keybind: Arc::new(AtomicBool::new(false)),
    }
  }

  pub fn send(&self, message: BridgeMessage) {
    if let Some(sender) = &self.transport_sender {
      sender.send(message).unwrap_or_default();
    }
  }

  pub fn notify(&mut self, notification: Notification) {
    if notification.message_id.is_some()
      && self
        .chat_messages
        .iter()
        .chain(&self.pending_chat)
        .any(|chat| chat.message_id == notification.message_id)
    {
      return;
    }

    self.pending_messages.push_back(notification);
    self.show_pending();
  }

  pub fn push_chat(&mut self, chat: Notification) -> bool {
    if self.pending_chat.len() >= MAX_PENDING_CHAT {
      self.pending_chat.pop_front();
    }
    self.pending_chat.push_back(chat);
    self.show_pending()
  }

  pub fn show_pending(&mut self) -> bool {
    let max = self.max_rows();
    let now = chrono::Utc::now().timestamp();
    let mut shown = false;

    while self.messages.len() + self.chat_messages.len() < max {
      let (mut row, rows) = if let Some(row) = self.pending_messages.pop_front() {
        (row, &mut self.messages)
      } else if let Some(row) = self.pending_chat.pop_front() {
        (row, &mut self.chat_messages)
      } else {
        break;
      };

      if row.timestamp.is_some() {
        row.timestamp = Some(now);
      }
      rows.push(row);
      shown = true;
    }

    shown
  }

  pub fn max_rows(&self) -> usize {
    self.config.max_notifications.clamp(1, 10) as usize
  }
}
