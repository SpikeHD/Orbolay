use crate::{config::Config, payloads::MessageNotification, user::User};

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
  pub config: Config,
  pub is_open: bool,
  pub voice_users: Vec<User>,
  pub messages: Vec<MessageNotification>,
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
    }
  }
}
