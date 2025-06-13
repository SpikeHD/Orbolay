#[derive(Debug, Clone)]
pub struct CornerAlignment {
  pub top: bool,
  pub left: bool,
}

#[derive(Debug, Clone)]
pub struct Config {
  pub port: u16,
  pub message_alignment: CornerAlignment,
  pub user_alignment: CornerAlignment,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      port: 6888,
      message_alignment: CornerAlignment {
        top: true,
        left: false,
      },
      user_alignment: CornerAlignment {
        top: true,
        left: true,
      },
    }
  }
}