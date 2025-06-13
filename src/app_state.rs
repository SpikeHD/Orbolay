use crate::{payloads::MessageNotification, user::User};

#[derive(Debug, Clone)]
pub struct AppState {
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
      voice_users: vec![],
      messages: vec![],
    }
  }
}
