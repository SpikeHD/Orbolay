use display_info::DisplayInfo;
use freya::prelude::*;

use crate::{
  app_state::SharedAppState,
  config::{Config, TransportMode, load_config, save_config},
  util::colors::{GRAY, MUTED_GRAY, TRANSPARENT, from_tuple, to_tuple},
};

#[cfg(not(target_os = "macos"))]
use crate::keys::bind::{DEFAULT_OVERLAY_TOGGLE, keys_to_strings, strings_to_keys};

use setting::{SettingChange, SettingKind, SettingRow};

mod color_picker;
mod dropdown;
mod input;
#[cfg(not(target_os = "macos"))]
mod keybind;
mod setting;
mod toggle;

const WIDTH: f32 = 600.;
const HEIGHT: f32 = 600.;

const TRANSPORT_MODES: &[&str] = &["ipc", "websocket"];

const ALIGNMENTS: &[&str] = &[
  "topleft",
  "topright",
  "bottomleft",
  "bottomright",
  "topcenter",
  "bottomcenter",
  "centerleft",
  "centerright",
];

const VOICE_DISPLAY_OPTIONS: &[&str] =
  &["always", "always (semi-transparent)", "only when speaking"];

pub fn open_configurator(shared: SharedAppState, redraw_tx: flume::Sender<()>) {
  spawn(async move {
    let _ = Platform::get()
      .launch_window(configurator_window(shared, redraw_tx))
      .await;
  });
}

pub fn open_configurator_standalone() {
  // Basically a blocking, standalone version of open_configurator
  let shared = SharedAppState::default();
  if let Some(saved) = load_config() {
    shared.write().unwrap().config = saved;
  }
  let (redraw_tx, _) = flume::unbounded();

  launch(LaunchConfig::new().with_window(configurator_window(shared.clone(), redraw_tx)));
}

fn configurator_window(shared: SharedAppState, redraw_tx: flume::Sender<()>) -> WindowConfig {
  WindowConfig::new(move || configurator(shared.clone(), redraw_tx.clone()))
    .with_background(GRAY)
    .with_size(WIDTH as f64, HEIGHT as f64)
    .with_title("Orbolay Configurator")
    .with_resizable(false)
}

fn make_updater(
  shared: SharedAppState,
  redraw_tx: flume::Sender<()>,
  mut local_config: State<Config>,
  update_fn: impl Fn(&mut Config, String) + 'static,
) -> EventHandler<SettingChange> {
  EventHandler::new(move |change: SettingChange| {
    if let SettingChange::Value(value) = change {
      let updated = {
        let mut state = shared.write().unwrap();
        update_fn(&mut state.config, value);
        state.config.clone()
      };
      save_config(&updated);
      local_config.set(updated);
      redraw_tx.send(()).ok();
    }
  })
}

fn make_color_updater(
  shared: SharedAppState,
  redraw_tx: flume::Sender<()>,
  mut local_config: State<Config>,
  update_fn: impl Fn(&mut Config, (u8, u8, u8)) + 'static,
) -> EventHandler<SettingChange> {
  EventHandler::new(move |change: SettingChange| {
    if let SettingChange::Color(color) = change {
      let updated = {
        let mut state = shared.write().unwrap();
        update_fn(&mut state.config, to_tuple(color));
        state.config.clone()
      };
      save_config(&updated);
      local_config.set(updated);
      redraw_tx.send(()).ok();
    }
  })
}

#[cfg(not(target_os = "macos"))]
fn make_keybind_updater(
  shared: SharedAppState,
  redraw_tx: flume::Sender<()>,
  mut local_config: State<Config>,
  update_fn: impl Fn(&mut Config, Vec<rdev::Key>) + 'static,
) -> EventHandler<SettingChange> {
  EventHandler::new(move |change: SettingChange| {
    if let SettingChange::Keys(keys) = change {
      let updated = {
        let mut state = shared.write().unwrap();
        update_fn(&mut state.config, keys);
        state.config.clone()
      };
      save_config(&updated);
      local_config.set(updated);
      redraw_tx.send(()).ok();
    }
  })
}

fn configurator(shared: SharedAppState, redraw_tx: flume::Sender<()>) -> impl IntoElement {
  use_init_theme(dark_theme);

  // Make the recording flag available to KeybindControl
  let recording_flag = shared.read().unwrap().recording_keybind.clone();
  use_provide_context(move || recording_flag);

  let local_config = use_state(|| shared.read().unwrap().config.clone());
  let config = local_config.read().clone();

  let all_displays = DisplayInfo::all().unwrap_or_default();
  let display_names: Vec<String> = all_displays
    .iter()
    .map(|d| format!("{} ({}x{})", d.friendly_name.clone(), d.width, d.height))
    .collect();
  let display_names_for_update = display_names.clone();

  let inner = rect()
    .direction(Direction::Vertical)
    .width(Size::fill())
    .padding(Gaps::new_symmetric(0., 16.))
    .child(SettingRow {
      name: "Connection Mode".into(),
      description: Some(
        "Set the communication method between Orbolay and Discord. If using an offical client, use \"ipc\", otherwise, use \"websocket\"."
          .into(),
      ),
      kind: SettingKind::Dropdown(
        TRANSPORT_MODES.iter().map(|s| s.to_string()).collect(),
        Some(config.transport_mode.to_string()),
      ),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        cfg.transport_mode = v.into();
      }),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Websocket Port".into(),
      description: Some("Port the websocket server listens on (websocket mode only). REQUIRES RESTART.".into()),
      kind: SettingKind::Input(Some(config.port.unwrap_or(6888).to_string())),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        if let Ok(n) = v.trim().parse::<u16>() {
          cfg.port = Some(n);
        }
      }),
      disabled: config.transport_mode != TransportMode::Websocket,
    })
    .child(divider());

  #[cfg(not(target_os = "macos"))]
  let inner = inner
    .child(SettingRow {
      name: "Enable Keybind".into(),
      description: Some("Toggle overlay visibility with a keybind".into()),
      kind: SettingKind::Toggle(config.is_keybind_enabled.unwrap_or(true)),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        cfg.is_keybind_enabled = Some(v == "true");
      }),
      disabled: false,
    })
    .child(divider());

  #[cfg(not(target_os = "macos"))]
  let inner = inner
    .child(SettingRow {
      name: "Overlay Keybind".into(),
      description: Some("The keybind used to open the overlay".into()),
      kind: SettingKind::Keybind(Some(strings_to_keys(
        config
          .overlay_keybind
          .clone()
          .unwrap_or_else(|| DEFAULT_OVERLAY_TOGGLE.clone()),
      ))),
      on_change: make_keybind_updater(
        shared.clone(),
        redraw_tx.clone(),
        local_config,
        |cfg, keys| {
          cfg.overlay_keybind = Some(keys_to_strings(keys));
        },
      ),
      disabled: !config.is_keybind_enabled.unwrap_or(true),
    })
    .child(divider());

  let inner = inner
    .child(SettingRow {
      name: "Display".into(),
      description: Some("The display to show the overlay on".into()),
      kind: SettingKind::Dropdown(
        display_names.clone(),
        config
          .display_idx
          .and_then(|i| display_names.get(i).cloned()),
      ),
      on_change: make_updater(
        shared.clone(),
        redraw_tx.clone(),
        local_config,
        move |cfg, v| {
          if let Some(idx) = display_names_for_update.iter().position(|name| name == &v) {
            cfg.display_idx = Some(idx);
          }
        },
      ),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Accent Color".into(),
      description: Some("The accent color for the overlay".into()),
      kind: SettingKind::Color(from_tuple(config.accent)),
      on_change: make_color_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        cfg.accent = v;
      }),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Text Color".into(),
      description: Some("The text color for the overlay".into()),
      kind: SettingKind::Color(from_tuple(config.text_color)),
      on_change: make_color_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        cfg.text_color = v;
      }),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Force Software Renderer".into(),
      description: Some(
        "Use software rendering instead of hardware acceleration. Requires restart.".into(),
      ),
      kind: SettingKind::Toggle(config.software_rendering.unwrap_or(false)),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        cfg.software_rendering = Some(v == "true");
      }),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Display Voice Members".into(),
      description: Some("Control when and how voice users are visible".into()),
      kind: SettingKind::Dropdown(
        VOICE_DISPLAY_OPTIONS
          .iter()
          .map(|s| s.to_string())
          .collect(),
        Some(
          config
            .display_voice_members
            .clone()
            .unwrap_or_default()
            .to_string(),
        ),
      ),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        cfg.display_voice_members = Some(v.into());
      }),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Semi-Transparent Notifications".into(),
      description: Some("Fade notifications when the overlay is closed".into()),
      kind: SettingKind::Toggle(config.messages_semitransparent),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        cfg.messages_semitransparent = v == "true";
      }),
      disabled: false,
    })
    .child(divider());

  let inner = inner
    .child(SettingRow {
      name: "Voice Alignment".into(),
      description: Some("Screen position for voice users".into()),
      kind: SettingKind::Dropdown(
        ALIGNMENTS.iter().map(|s| s.to_string()).collect(),
        config
          .user_alignment
          .clone()
          .or_else(|| Some("topleft".into())),
      ),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        cfg.user_alignment = Some(v);
      }),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Voice X Offset (px)".into(),
      description: None,
      kind: SettingKind::Input(Some(config.user_offset_x.to_string())),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        if let Ok(n) = v.trim().parse::<i32>() {
          cfg.user_offset_x = n;
        }
      }),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Voice Y Offset (px)".into(),
      description: None,
      kind: SettingKind::Input(Some(config.user_offset_y.to_string())),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        if let Ok(n) = v.trim().parse::<i32>() {
          cfg.user_offset_y = n;
        }
      }),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Notification Alignment".into(),
      description: Some("Screen position for notifications".into()),
      kind: SettingKind::Dropdown(
        ALIGNMENTS.iter().map(|s| s.to_string()).collect(),
        config
          .message_alignment
          .clone()
          .or_else(|| Some("topright".into())),
      ),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        cfg.message_alignment = Some(v);
      }),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Messages X Offset (px)".into(),
      description: None,
      kind: SettingKind::Input(Some(config.message_offset_x.to_string())),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        if let Ok(n) = v.trim().parse::<i32>() {
          cfg.message_offset_x = n;
        }
      }),
      disabled: false,
    })
    .child(divider())
    .child(SettingRow {
      name: "Messages Y Offset (px)".into(),
      description: None,
      kind: SettingKind::Input(Some(config.message_offset_y.to_string())),
      on_change: make_updater(shared.clone(), redraw_tx.clone(), local_config, |cfg, v| {
        if let Ok(n) = v.trim().parse::<i32>() {
          cfg.message_offset_y = n;
        }
      }),
      disabled: false,
    })
    .child(
      label()
        .text("Press \"C\" with the overlay open to open this window again!")
        .color(MUTED_GRAY)
        .font_size(12.)
        .margin(16.)
        .text_align(TextAlign::Center)
        .width(Size::fill()),
    );

  rect()
    .width(Size::fill())
    .height(Size::fill())
    .background(GRAY)
    .direction(Direction::Vertical)
    .child(
      ScrollView::new()
        .height(Size::fill())
        .width(Size::fill())
        .direction(Direction::Vertical)
        .child(inner),
    )
}

fn divider() -> impl IntoElement {
  rect()
    .width(Size::fill())
    .height(Size::px(1.))
    .padding(16.)
    .background(TRANSPARENT)
}
