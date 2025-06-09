#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use freya::prelude::*;
use gumdrop::Options;
use winit::{dpi::LogicalPosition, window::WindowLevel};

use crate::{
  components::user_row::user_row,
  user::{User, UserVoiceState},
};

mod components;
mod logger;
mod user;
mod websocket;

const GIT_HASH: Option<&str> = option_env!("GIT_HASH");
const APP_NAME: Option<&str> = option_env!("CARGO_PKG_NAME");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

pub static STATE: GlobalSignal<AppState> = GlobalSignal::new(|| AppState {
  voice_users: vec![],
});

#[derive(Debug, Clone)]
pub struct AppState {
  pub voice_users: Vec<User>,
}

#[derive(Debug, Clone, Options)]
pub struct Args {
  #[options(help = "The port to run the websocket server on", default = "1981")]
  port: u16,

  #[options(help = "Print version information")]
  version: bool,

  #[options(help = "Enable various debugging features")]
  debug: bool,
}

fn main() {
  let args = Args::parse_args_default_or_exit();

  if args.version {
    println!(
      "{} version {} (rev {})",
      APP_NAME.unwrap_or("Unknown"),
      APP_VERSION.unwrap_or("0.0.0"),
      GIT_HASH.unwrap_or("unknown")
    );
    std::process::exit(0);
  }

  std::thread::spawn(move || {
    websocket::create_websocket(args.port).expect("Failed to start websocket server");
  });

  launch_cfg(
    app,
    LaunchConfig::<f32>::new()
      .with_decorations(false)
      .with_background("transparent")
      .with_transparency(true)
      .with_window_attributes(|w| {
        w.with_window_level(WindowLevel::AlwaysOnTop)
          .with_resizable(false)
          .with_position(LogicalPosition::new(0, 0))
      }),
  );
}

fn app() -> Element {
  let args = Args::parse_args_default_or_exit();
  let platform = use_platform();

  platform.with_window(move |w| {
    // Disable hittest
    if !args.debug {
      w.set_cursor_hittest(false).unwrap_or_default();
    }
  });

  rsx!(
    rect {
      content: "flex",
      direction: "vertical",

      background: "transparent",
      height: "600",
      width: "200",

      user_row {
        user: User {
          name: "John Doe".to_string(),
          avatar: vec![1],
          voice_state: UserVoiceState::Speaking,
        }
      }

      user_row {
        user: User {
          name: "Jane Doe".to_string(),
          avatar: vec![1],
          voice_state: UserVoiceState::NotSpeaking,
        }
      }

      user_row {
        user: User {
          name: "Jebediah Doe".to_string(),
          avatar: vec![1],
          voice_state: UserVoiceState::Muted,
        }
      }
    }
  )
}
