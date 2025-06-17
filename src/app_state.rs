use crate::{config::Config, payloads::MessageNotification, user::User, websocket::BridgeMessage};

#[derive(Debug, Clone)]
pub struct AppState {
  pub config: Config,
  pub is_open: bool,
  pub voice_users: Vec<User>,
  pub messages: Vec<MessageNotification>,

  pub ws_sender: Option<flume::Sender<BridgeMessage>>,
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
      is_open: false,
      voice_users: vec![],
      messages: vec![],

      ws_sender: None,
    }
  }

  pub fn send(&mut self, message: BridgeMessage) {
    if let Some(sender) = &self.ws_sender {
      sender.send(message).unwrap_or_default();
    }
  }
}
