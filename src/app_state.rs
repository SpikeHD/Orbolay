use crate::user::User;

#[derive(Debug, Clone)]
pub struct AppState {
  pub voice_users: Vec<User>,
}

impl AppState {
  pub fn new() -> Self {
    Self {
      voice_users: vec![],
    }
  }
}
