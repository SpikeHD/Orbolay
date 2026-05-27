#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
#![allow(clippy::borrow_interior_mutable_const)]
#![allow(clippy::declare_interior_mutable_const)]

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
  components::{MessagesSection, VoiceControls, VoiceSection},
  config::{is_first_run, load_config, save_config},
  config_watcher::start_config_watcher,
  configurator::{open_configurator, open_configurator_standalone},
  display::{specific_monitor_or_primary, update_monitor},
  manager::OverlayManager,
  notifications::create_notification_thread,
  payloads::MessageNotification,
  transport::create_transport_thread,
  updates::maybe_notify_update,
  util::colors,
};

mod app_state;
mod components;
mod config;
mod config_watcher;
mod configurator;
mod display;
mod ipc;
#[cfg(not(target_os = "macos"))]
mod keys;
mod logger;
mod manager;
mod notifications;
mod payloads;
mod transport;
mod updates;
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

  #[options(help = "Force IPC mode instead of websocket")]
  ipc: bool,

  #[options(help = "Open the configuration window")]
  config: bool,
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

  if args.config {
    open_configurator_standalone();
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

  let display = specific_monitor_or_primary();

  let monitor_position = (display.x, display.y);
  let monitor_size = (display.width, display.height);

  #[cfg(target_os = "macos")]
  let window_size = (
    (monitor_size.0 + 1) as f64 * display.scale_factor as f64,
    (monitor_size.1 + 1) as f64 * display.scale_factor as f64,
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
    #[cfg(not(target_os = "macos"))]
    let (keybind_tx, keybind_rx) = flume::unbounded::<keys::KeyEvent>();

    app_state.write().ws_sender = Some(ws_sender);

    // Shared state for background threads
    let mut initial = AppState::new();

    if let Some(saved) = load_config() {
      initial.config = saved;
    }

    let shared: SharedAppState = std::sync::Arc::new(std::sync::RwLock::new(initial));

    if !args.debug {
      Platform::get().with_window(None, |w| {
        w.set_cursor_hittest(false).unwrap_or_default();
      });
    }

    #[cfg(not(target_os = "macos"))]
    keys::watch_keybinds(shared.clone(), keybind_tx);

    create_transport_thread(shared.clone(), redraw_tx.clone(), args, ws_receiver);
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
    let shared_sync = shared.clone();
    spawn_forever(async move {
      while let Ok(()) = redraw_rx.recv_async().await {
        let synced = shared_sync.read().unwrap().clone();
        let ws_sender = app_state.read().ws_sender.clone();
        let is_open = app_state.read().is_open;
        *app_state.write() = AppState {
          ws_sender,
          is_open,
          ..synced
        };

        update_monitor();
      }
    });

    // Both of these must happen before shared/redraw_tx are moved into the keybind handler
    if is_first_run() {
      open_configurator(shared.clone(), redraw_tx.clone());
      redraw_tx.send(()).ok();

      // Write the config regardless so we don't trigger this in the future
      {
        let state = shared.read().unwrap();
        save_config(&state.config);
      }
    }

    start_config_watcher(shared.clone(), redraw_tx.clone());
    maybe_notify_update(shared.clone());

    #[cfg(not(target_os = "macos"))]
    spawn_forever(async move {
      while let Ok(event) = keybind_rx.recv_async().await {
        match event {
          keys::KeyEvent::ToggleOverlay => {
            let current = app_state.read().is_open;
            app_state.write().is_open = !current;
          }
          keys::KeyEvent::OpenConfigurator if app_state.read().is_open => {
            open_configurator(shared.clone(), redraw_tx.clone());
            app_state.write().is_open = false;
          }
          _ => {}
        }
      }
    });
  });

  // Sync is_open -> cursor hit-test
  use_side_effect(move || {
    let is_open = app_state.read().is_open;
    Platform::get().with_window(None, move |w| {
      let _ = w.set_cursor_hittest(is_open);
    });
  });

  let state = app_state.read();
  let voice_users = state.voice_users.clone();
  let messages = state.messages.clone();
  let is_open = state.is_open;
  let is_censor = state.is_censor;
  let config = state.config.clone();
  let current_user = state
    .voice_users
    .iter()
    .find(|u| u.id == state.config.user_id)
    .cloned();
  drop(state);

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
    .child(VoiceSection {
      voice_users,
      is_open,
      is_censor,
      user_alignment: config
        .user_alignment
        .clone()
        .unwrap_or_else(|| "topleft".into()),
      user_offset_x: config.user_offset_x,
      user_offset_y: config.user_offset_y,
      voice_semitransparent: config.voice_semitransparent.unwrap_or(true),
    })
    // Messages
    .child(MessagesSection {
      messages,
      is_open,
      is_censor,
      message_alignment: config
        .message_alignment
        .clone()
        .unwrap_or_else(|| "topright".into()),
      message_offset_x: config.message_offset_x,
      message_offset_y: config.message_offset_y,
      messages_semitransparent: config.messages_semitransparent,
      app_state,
    })
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
