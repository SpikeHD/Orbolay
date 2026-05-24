#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use display_info::DisplayInfo;
use freya::prelude::*;
use gumdrop::Options;
use native_dialog::{MessageDialogBuilder, MessageLevel};
#[cfg(target_os = "windows")]
use winit::platform::windows::WindowAttributesExtWindows;
use winit::{
  dpi::{PhysicalPosition, PhysicalSize},
  window::WindowLevel,
};

use crate::{
  app_state::{AppState, SharedAppState},
  components::{MessageRow, UserRow, VoiceControls},
  config::CornerAlignment,
  manager::OverlayManager,
  notifications::create_notification_thread,
  payloads::MessageNotification,
  transport::create_transport_thread,
  util::{colors, text::censor},
};

mod app_state;
mod components;
mod config;
mod ipc;
#[cfg(not(target_os = "macos"))]
mod keys;
mod logger;
mod manager;
mod notifications;
mod payloads;
mod transport;
mod user;
mod util;
mod websocket;

const GIT_HASH: Option<&str> = option_env!("GIT_HASH");
const APP_NAME: Option<&str> = option_env!("CARGO_PKG_NAME");
const APP_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
const CLIENT_ID: &str = "207646673902501888";

#[derive(Debug, Clone, Options)]
pub struct Args {
  #[options(help = "Display usage information")]
  help: bool,

  #[options(help = "The port to run the websocket server on", default = "6888")]
  port: u16,

  #[options(help = "Print version information")]
  version: bool,

  #[options(help = "Enable various debugging features")]
  debug: bool,

  #[options(help = "Force websocket mode instead of IPC")]
  websocket: bool,
}

fn main() {
  let args = Args::parse_args_default_or_exit();

  if args.help_requested() {
    println!("{}", Args::usage());
    std::process::exit(0);
  }

  if args.version {
    println!(
      "{} version {} (rev {})",
      APP_NAME.unwrap_or("Unknown"),
      APP_VERSION.unwrap_or("0.0.0"),
      GIT_HASH.unwrap_or("unknown")
    );
    std::process::exit(0);
  }

  if util::process::is_already_running() {
    MessageDialogBuilder::default()
      .set_level(MessageLevel::Error)
      .set_title("Orbolay")
      .set_text("Orbolay is already running. Kill the existing process before starting a new one.")
      .alert()
      .show()
      .expect("Failed to show message dialog");
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
    (monitor_size.0 + 1) as f64 * primary.scale_factor as f64,
    (monitor_size.1 + 1) as f64 * primary.scale_factor as f64,
  );
  #[cfg(not(target_os = "macos"))]
  let window_size = ((monitor_size.0 + 1) as f64, (monitor_size.1 - 1) as f64);

  #[cfg(target_os = "linux")]
  {
    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    if session_type.to_lowercase() == "wayland" {
      warn!(
        "You are using Wayland. Orbolay will probably not work correctly unless running with XWayland."
      );
    }
  }

  launch(
    LaunchConfig::new().with_window(
      WindowConfig::new(app)
        .with_title("orbolay")
        .with_decorations(false)
        .with_transparency(true)
        .with_background(Color::TRANSPARENT)
        .with_window_attributes(move |mut w, _event_loop| {
          w = w
            .with_inner_size(PhysicalSize::new(window_size.0, window_size.1))
            .with_resizable(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(PhysicalPosition::new(
              monitor_position.0,
              monitor_position.1,
            ));

          #[cfg(target_os = "windows")]
          {
            w = w.with_skip_taskbar(true);
          }

          #[cfg(target_os = "linux")]
          {
            use winit::platform::wayland::WindowAttributesExtWayland;
            use winit::platform::x11::{WindowAttributesExtX11, WindowType};

            w = WindowAttributesExtX11::with_name(w, "orbolay", "orbolay")
              .with_x11_window_type(vec![WindowType::Utility])
              .with_override_redirect(true);
            w = WindowAttributesExtWayland::with_name(w, "orbolay", "orbolay");
          }

          w
        }),
    ),
  );
}

fn app() -> impl IntoElement {
  let args = Args::parse_args_default_or_exit();
  let mut app_state = use_state(AppState::new);

  use_hook(move || {
    let (ws_sender, ws_receiver) = flume::unbounded::<crate::util::bridge::BridgeMessage>();
    let (redraw_tx, redraw_rx) = flume::unbounded::<()>();
    let (overlay_tx, overlay_rx) = flume::unbounded::<()>();

    app_state.write().ws_sender = Some(ws_sender);

    // Shared state for background threads
    let shared: SharedAppState = std::sync::Arc::new(std::sync::RwLock::new(AppState::new()));

    if !args.debug {
      Platform::get().with_window(None, |w| {
        w.set_cursor_hittest(false).unwrap_or_default();
      });
    }

    #[cfg(not(target_os = "macos"))]
    keys::watch_keybinds(shared.clone(), overlay_tx);

    create_transport_thread(shared.clone(), redraw_tx.clone(), args.clone(), ws_receiver);
    create_notification_thread(shared.clone(), redraw_tx.clone());

    shared.write().unwrap().notify(MessageNotification {
      title: format!(
        "Orbolay v{} (rev {})",
        APP_VERSION.unwrap_or("0.0.0"),
        GIT_HASH.unwrap_or("unknown")
      ),
      body: "by SpikeHD".to_string(),
      timestamp: Some(chrono::Utc::now().timestamp().to_string()),
      icon: "https://avatars.githubusercontent.com/u/25207995?v=4".to_string(),
      guild_id: None,
      channel_id: None,
      message_id: None,
    });

    // sync SharedAppState -> AppState on every redraw signal
    spawn_forever(async move {
      while let Ok(()) = redraw_rx.recv_async().await {
        let synced = shared.read().unwrap().clone();
        let ws_sender = app_state.read().ws_sender.clone();
        let is_open = app_state.read().is_open;
        *app_state.write() = AppState {
          ws_sender,
          is_open,
          ..synced
        };
      }
    });

    // toggle is_open in AppState when keybind fires
    spawn_forever(async move {
      while let Ok(()) = overlay_rx.recv_async().await {
        let current = app_state.read().is_open;
        app_state.write().is_open = !current;
      }
    });

    redraw_tx.send(()).ok();
  });

  // Sync is_open -> cursor hit-test
  use_side_effect(move || {
    let is_open = app_state.read().is_open;
    Platform::get().with_window(None, move |w| {
      let _ = w.set_cursor_hittest(is_open);
    });
  });

  let state = app_state.read();
  let is_open = state.is_open;
  let is_censor = state.is_censor;
  let config = state.config.clone();
  let voice_users = state.voice_users.clone();
  let messages = state.messages.clone();
  let current_user = state
    .voice_users
    .iter()
    .find(|u| u.id == state.config.user_id)
    .cloned();

  let user_alignment = CornerAlignment::from_str(&config.user_alignment);
  let msg_alignment = CornerAlignment::from_str(&config.message_alignment);

  let user_gaps = user_alignment.to_gaps(config.user_offset_x, config.user_offset_y);
  let msg_gaps = msg_alignment.to_gaps(config.message_offset_x, config.message_offset_y);

  // Root container
  let voice_section = voice_users.iter().fold(
    rect()
      .direction(Direction::Vertical)
      .cross_align(user_alignment.x.to_freya())
      .main_align(user_alignment.y.to_freya())
      .position(Position::new_absolute().top(0.).left(0.))
      .background(Color::TRANSPARENT)
      .height(Size::fill())
      .width(Size::fill())
      .padding(user_gaps),
    |el, user| {
      let mut u = user.clone();
      if is_censor {
        u.name = censor(&u.name);
      }
      el.child(UserRow { user: u, app_state })
    },
  );

  let messages_section = messages.iter().fold(
    rect()
      .direction(Direction::Vertical)
      .cross_align(msg_alignment.x.to_freya())
      .main_align(msg_alignment.y.to_freya())
      .position(Position::new_absolute().top(0.).left(0.))
      .background(Color::TRANSPARENT)
      .height(Size::fill())
      .width(Size::fill())
      .padding(msg_gaps)
      .opacity(if config.messages_semitransparent && !is_open {
        0.5
      } else {
        1.0
      }),
    |el, message| {
      if is_censor {
        el
      } else {
        el.child(MessageRow {
          app_state,
          message: message.clone(),
        })
      }
    },
  );

  rect()
    .width(Size::fill())
    .height(Size::fill())
    // Background overlay
    .child(
      rect()
        .position(Position::new_absolute().top(0.).left(0.))
        .background(if is_open {
          colors::TRANSPARENT_GRAY
        } else {
          Color::TRANSPARENT
        })
        .width(Size::fill())
        .height(Size::fill())
        .on_press(move |_| {
          OverlayManager::close(app_state);
        }),
    )
    // Voice users
    .child(voice_section)
    // Messages
    .child(messages_section)
    // Voice controls
    .maybe(is_open, |el| {
      el.maybe_child(current_user.map(|user| {
        rect()
          .position(Position::new_absolute().top(0.).left(0.))
          .direction(Direction::Horizontal)
          .main_align(Alignment::Center)
          .cross_align(Alignment::End)
          .height(Size::percent(90.))
          .width(Size::fill())
          .child(VoiceControls { user, app_state })
      }))
    })
}
