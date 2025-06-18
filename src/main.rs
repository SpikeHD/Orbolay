#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use std::collections::HashMap;

use display_info::DisplayInfo;
use freya::prelude::*;
use gumdrop::Options;
#[cfg(target_os = "windows")]
use winit::platform::windows::WindowAttributesExtWindows;
use winit::{
  dpi::{PhysicalPosition, PhysicalSize},
  window::WindowLevel,
};

use crate::{
  app_state::AppState,
  components::{message_row::message_row, user_row::user_row, voice_controls::voice_controls},
};

mod app_state;
mod components;
mod config;
#[cfg(target_os = "windows")]
mod keys;
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
  let primary = displays
    .iter()
    .find(|m| m.is_primary)
    .unwrap_or(displays.first().expect("No displays found"));
  let monitor_position = (primary.x, primary.y);
  let monitor_size = (primary.width, primary.height);

  #[cfg(target_os = "macos")]
  let window_size = (
    (monitor_size.0 + 1) as f32 * primary.scale_factor,
    (monitor_size.1 + 1) as f32 * primary.scale_factor,
  );
  #[cfg(not(target_os = "macos"))]
  let window_size = (monitor_size.0 + 1, monitor_size.1 + 1);

  launch_cfg(
    app,
    LaunchConfig::<f32>::new()
      .with_decorations(false)
      .with_background("transparent")
      .with_transparency(true)
      .with_window_attributes(move |w| {
        #[cfg(target_os = "windows")]
        return w
            .with_skip_taskbar(true)
            // https://discourse.glfw.org/t/black-screen-when-setting-window-to-transparent-and-size-to-1920x1080/2585/4
            .with_inner_size(PhysicalSize::new(window_size.0, window_size.1))
            .with_resizable(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(PhysicalPosition::new(
              monitor_position.0,
              monitor_position.1,
            ));

        #[cfg(not(target_os = "windows"))]
        return w
            // https://discourse.glfw.org/t/black-screen-when-setting-window-to-transparent-and-size-to-1920x1080/2585/4
            .with_inner_size(PhysicalSize::new(window_size.0, window_size.1))
            .with_resizable(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(PhysicalPosition::new(
              monitor_position.0,
              monitor_position.1,
            ));
      }),
  );
}

fn app() -> Element {
  let args = Args::parse_args_default_or_exit();
  let platform = use_platform();
  let mut app_state = use_signal_sync(AppState::new);

  use_effect(move || {
    let (ws_sender, ws_receiver) = flume::unbounded();
    app_state.write().ws_sender = Some(ws_sender);

    platform.with_window(move |w| {
      if !args.debug {
        w.set_cursor_hittest(false).unwrap_or_default();
      }
    });

    std::thread::spawn(move || {
      websocket::create_websocket(args.port, app_state, ws_receiver)
        .expect("Failed to start websocket server");
    });

    #[cfg(target_os = "windows")]
    keys::watch_keybinds(app_state, platform.sender());

    // Check the messages once per second, removing any that are older than 5 seconds
    std::thread::spawn(move || {
      loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let current_timestamp = chrono::Utc::now().timestamp();
        app_state.write().messages.retain(|message| {
          let msg = message.clone();
          if let Some(message_timestamp) = msg.timestamp {
            let timestamp = message_timestamp.parse::<i64>().unwrap_or(0);
            return current_timestamp - timestamp < 5;
          }

          true
        });
      }
    });
  });

  rsx!(
    // Background layer
    rect {
      position: "absolute",
      position_top: "0",
      position_left: "0",

      background: if app_state.read().is_open { "#555555" } else { "transparent" },
      width: "100%",
      height: "100%",
      opacity: "0.3",
    }

    // Voice users
    rect {
      content: "flex",
      direction: "vertical",
      cross_align: if app_state.read().config.user_alignment.left { "start" } else { "end" },
      main_align: if app_state.read().config.user_alignment.top { "start" } else { "end" },

      position: "absolute",
      position_top: "0",
      position_left: "0",

      background: "transparent",
      height: "100%",
      width: "100%",

      for user in app_state.read().voice_users.iter() {
        user_row {
          user: user.clone(),
          app_state: app_state
        }
      }
    }

    // Messages
    rect {
      content: "flex",
      direction: "vertical",
      cross_align: if app_state.read().config.message_alignment.left { "start" } else { "end" },
      main_align: if app_state.read().config.message_alignment.top { "start" } else { "end" },

      position: "absolute",
      position_top: "0",
      position_left: "0",

      background: "transparent",
      height: "100%",
      width: "100%",

      opacity: if app_state.read().config.messages_semitransparent && !app_state.read().is_open { "0.5" } else { "1.0" },

      for message in app_state.read().messages.iter() {
        message_row {
          message: message.clone()
        }
      }
    }

    // Voice Controls
    if app_state.read().is_open {
      if let Some(user) = app_state.read().voice_users.iter().find(|u| u.id == app_state.read().config.user_id) {
        rect {
          position: "absolute",
          position_top: "0",
          position_left: "0",

          content: "flex",
          direction: "horizontal",
          main_align: "center",
          cross_align: "end",
          height: "90%",
          width: "100%",

          voice_controls {
            user: user.clone(),
            app_state: app_state
          }
        }
      }
    }
  )
}
