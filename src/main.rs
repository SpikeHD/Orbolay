#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::collections::HashMap;

use display_info::DisplayInfo;
use freya::prelude::*;
use gumdrop::Options;
use winit::{dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize}, window::WindowLevel};

use crate::{app_state::AppState, components::user_row::user_row};

mod app_state;
mod components;
mod logger;
mod payloads;
mod user;
mod util;
mod websocket;

const GIT_HASH: Option<&str> = option_env!("GIT_HASH");
const APP_NAME: Option<&str> = option_env!("CARGO_PKG_NAME");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

pub static STATE: GlobalSignal<AppState> = GlobalSignal::new(AppState::new);
pub static AVATAR_CACHE: GlobalSignal<HashMap<String, Vec<u8>>> = GlobalSignal::new(HashMap::new);

#[derive(Debug, Clone, Options)]
pub struct Args {
  #[options(help = "The port to run the websocket server on", default = "6888")]
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

  let displays = DisplayInfo::all().expect("Failed to get display information");
  let primary = displays.iter().find(|m| m.is_primary).unwrap_or(displays.first().expect("No displays found"));
  let monitor_position = (primary.x, primary.y);
  let monitor_size = (primary.width, primary.height);

  log!("Displays: {:?}", displays);
  log!("Primary monitor: {:?}", primary);
  log!("Monitor position: {:?}", monitor_position);
  log!("Monitor size: {:?}", monitor_size);

  launch_cfg(
    app,
    LaunchConfig::<f32>::new()
      .with_decorations(false)
      .with_background("transparent")
      .with_transparency(true)
      .with_window_attributes(move |w| {
        w.with_window_level(WindowLevel::AlwaysOnTop)
          .with_inner_size(LogicalSize::new(monitor_size.0, monitor_size.1))
          .with_resizable(false)
          .with_position(LogicalPosition::new(monitor_position.0, monitor_position.1))
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

  let app_state = use_signal_sync(AppState::new);

  use_effect(move || {
    std::thread::spawn(move || {
      websocket::create_websocket(args.port, app_state).expect("Failed to start websocket server");
    });
  });

  rsx!(
    // Voice users
    rect {
      content: "flex",
      direction: "vertical",

      position: "absolute",
      position_top: "0",
      position_left: "0",

      background: "transparent",
      height: "100%",
      width: "30%",

      for user in app_state().voice_users.iter() {
        user_row {
          user: user.clone()
        }
      }
    }
  )
}
