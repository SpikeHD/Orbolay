use std::str::FromStr;

use display_info::DisplayInfo;
use freya::prelude::*;

use orbolay_core::{
  app_state::AppHandle,
  config::{Config, TransportMode, config_dir, save_config},
  dirs::{TEMPLATES_DIR, ensure_config_dirs},
  monitors::{MonitorInfo, monitors_list},
  payloads::Notification,
};
use orbolay_logging::warn;

use crate::util::theme::{DARKISH_GRAY, LIGHT_GRAY, SUPERLIGHT_GRAY, from_tuple, to_tuple};
use crate::util::{
  notification_template::list_notification_templates, user_template::list_user_templates,
};

#[cfg(not(target_os = "macos"))]
use orbolay_keys::{DEFAULT_OVERLAY_TOGGLE, keys_to_strings, strings_to_keys};

#[cfg(not(target_os = "macos"))]
use super::keybind::KeybindControl;
use super::secondary_button;
use super::{
  color_picker::ColorPickerControl, dropdown::DropdownControl, input::InputControl,
  setting::SettingRow, toggle::ToggleControl,
};

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

const UI_SCALE_OPTIONS: &[f32] = &[0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];

const DEFAULT_TEMPLATE: &str = "Default";

fn ui_scale_label(scale: f32) -> String {
  format!("{:.0}%", scale * 100.0)
}

fn to_options(values: &[&str]) -> Vec<String> {
  values.iter().map(|s| s.to_string()).collect()
}

fn option_index(options: &[&str], value: &str) -> usize {
  options
    .iter()
    .position(|option| *option == value)
    .unwrap_or(0)
}

fn open_templates_folder() {
  let Some(dir) = config_dir() else {
    warn!("Could not resolve the config directory");
    return;
  };
  let templates_dir = dir.join(TEMPLATES_DIR);

  ensure_config_dirs();

  if let Err(err) = open::that(&templates_dir) {
    warn!("Failed to open templates folder: {err}");
  }
}

fn available_displays() -> Vec<MonitorInfo> {
  let cached = monitors_list();
  if !cached.is_empty() {
    return cached;
  }

  DisplayInfo::all()
    .unwrap_or_default()
    .into_iter()
    .map(|d| MonitorInfo {
      name: d.friendly_name,
      is_primary: d.is_primary,
      width: d.width,
      height: d.height,
    })
    .collect()
}

pub fn apply_config(
  app: &AppHandle,
  mut local_config: State<Config>,
  update: impl FnOnce(&mut Config),
) {
  let updated = app.update(|state| {
    update(&mut state.config);
    state.config.clone()
  });

  save_config(&updated);
  local_config.set(updated);
}

fn wide_button(
  text: &'static str,
  on_press: impl Into<EventHandler<Event<PressEventData>>>,
) -> Element {
  rect()
    .width(Size::fill())
    .padding(Gaps::new_symmetric(8., 12.))
    .child(secondary_button(text, on_press))
    .into_element()
}

fn divider() -> impl IntoElement {
  rect()
    .width(Size::fill())
    .height(Size::px(1.0_f32))
    .background(LIGHT_GRAY)
}

fn card(rows: impl IntoIterator<Item = Element>) -> Element {
  let mut body = rect()
    .width(Size::fill())
    .margin(Gaps::new(0., 0., 20., 0.))
    .corner_radius(10.)
    .background(DARKISH_GRAY);

  for (idx, row) in rows.into_iter().enumerate() {
    if idx > 0 {
      body = body.child(divider());
    }
    body = body.child(row);
  }

  body.into_element()
}

fn group(title: &'static str, rows: impl IntoIterator<Item = Element>) -> Element {
  rect()
    .width(Size::fill())
    .child(
      label()
        .text(title)
        .color(SUPERLIGHT_GRAY)
        .font_size(12.)
        .font_weight(FontWeight::BOLD)
        .margin(Gaps::new(0., 0., 8., 4.)),
    )
    .child(card(rows))
    .into_element()
}

fn row(
  name: &str,
  description: Option<&str>,
  control: impl Into<Element>,
  disabled: bool,
) -> Element {
  SettingRow {
    name: name.into(),
    description: description.map(Into::into),
    control: control.into(),
    disabled,
  }
  .into()
}

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum Section {
  General,
  Voice,
  Notifications,
  Appearance,
  Connection,
  Advanced,
}

impl Section {
  pub const ALL: [Section; 6] = [
    Section::General,
    Section::Voice,
    Section::Notifications,
    Section::Appearance,
    Section::Connection,
    Section::Advanced,
  ];

  pub fn title(&self) -> &'static str {
    match self {
      Section::General => "General",
      Section::Voice => "Voice",
      Section::Notifications => "Notifications",
      Section::Appearance => "Appearance",
      Section::Connection => "Connection",
      Section::Advanced => "Advanced",
    }
  }

  pub fn description(&self) -> &'static str {
    match self {
      Section::General => "How the overlay opens and where it appears",
      Section::Voice => "Who is shown in the voice list, and where",
      Section::Notifications => "Popups for incoming messages and calls",
      Section::Appearance => "Colors, shapes and templates",
      Section::Connection => "How Orbolay connects to Discord",
      Section::Advanced => "Rendering options",
    }
  }
}

pub struct Settings {
  pub app: AppHandle,
  pub local_config: State<Config>,
  pub config: Config,
  pub standalone: bool,
}

impl Settings {
  pub fn groups(&self, section: Section) -> Vec<Element> {
    match section {
      Section::General => self.general(),
      Section::Voice => self.voice(),
      Section::Notifications => self.notifications(),
      Section::Appearance => self.appearance(),
      Section::Connection => self.connection(),
      Section::Advanced => self.advanced(),
    }
  }

  fn updater<T: 'static>(&self, update: impl Fn(&mut Config, T) + 'static) -> EventHandler<T> {
    let app = self.app.clone();
    let local_config = self.local_config;

    EventHandler::new(move |value| {
      apply_config(&app, local_config, |cfg| update(cfg, value));
    })
  }

  fn toggle(
    &self,
    name: &str,
    description: &str,
    value: bool,
    update: impl Fn(&mut Config, bool) + 'static,
  ) -> Element {
    row(
      name,
      Some(description),
      ToggleControl::new(value, self.updater(update)),
      false,
    )
  }

  fn dropdown(
    &self,
    name: &str,
    description: &str,
    options: Vec<String>,
    selected: usize,
    update: impl Fn(&mut Config, usize) + 'static,
  ) -> Element {
    row(
      name,
      Some(description),
      DropdownControl::new(options, selected, self.updater(update)),
      false,
    )
  }

  fn number<T: FromStr + ToString + 'static>(
    &self,
    name: &str,
    description: Option<&str>,
    value: T,
    disabled: bool,
    update: impl Fn(&mut Config, T) + 'static,
  ) -> Element {
    row(
      name,
      description,
      InputControl::new(
        value.to_string(),
        self.updater(move |cfg, text: String| {
          if let Ok(n) = text.trim().parse::<T>() {
            update(cfg, n);
          }
        }),
        !disabled,
      ),
      disabled,
    )
  }

  fn color(
    &self,
    name: &str,
    description: &str,
    value: (u8, u8, u8),
    update: impl Fn(&mut Config, Color) + 'static,
  ) -> Element {
    row(
      name,
      Some(description),
      ColorPickerControl::new(from_tuple(value), self.updater(update)),
      false,
    )
  }

  fn alignment(
    &self,
    name: &str,
    description: &str,
    value: &str,
    update: impl Fn(&mut Config, String) + 'static,
  ) -> Element {
    self.dropdown(
      name,
      description,
      to_options(ALIGNMENTS),
      option_index(ALIGNMENTS, value),
      move |cfg, idx| update(cfg, ALIGNMENTS[idx].to_string()),
    )
  }

  fn template(
    &self,
    name: &str,
    description: &str,
    templates: Vec<String>,
    value: Option<&str>,
    update: impl Fn(&mut Config, Option<String>) + 'static,
  ) -> Element {
    let selected = value
      .and_then(|value| templates.iter().position(|t| t == value))
      .map_or(0, |idx| idx + 1);
    let options = std::iter::once(DEFAULT_TEMPLATE.to_string())
      .chain(templates.iter().cloned())
      .collect();

    self.dropdown(name, description, options, selected, move |cfg, idx| {
      update(cfg, idx.checked_sub(1).map(|idx| templates[idx].clone()));
    })
  }

  #[cfg(not(target_os = "macos"))]
  fn keybind_group(&self) -> Element {
    let config = &self.config;
    let enabled = config.is_keybind_enabled.unwrap_or(true);
    let keys = strings_to_keys(
      config
        .overlay_keybind
        .clone()
        .unwrap_or_else(|| DEFAULT_OVERLAY_TOGGLE.get().map(String::from).to_vec()),
    );

    group(
      "Keybind",
      [
        self.toggle(
          "Enable Keybind",
          "Toggle overlay visibility with a keybind",
          enabled,
          |cfg, value| cfg.is_keybind_enabled = Some(value),
        ),
        row(
          "Overlay Keybind",
          Some("The keybind used to open the overlay"),
          KeybindControl::new(
            keys,
            self.updater(|cfg, keys| cfg.overlay_keybind = Some(keys_to_strings(keys))),
            enabled,
          ),
          !enabled,
        ),
      ],
    )
  }

  fn general(&self) -> Vec<Element> {
    let config = &self.config;

    let display_names: Vec<String> = available_displays()
      .iter()
      .map(|d| format!("{} ({}x{})", d.name, d.width, d.height))
      .collect();
    let selected_display = config
      .display_idx
      .filter(|idx| *idx < display_names.len())
      .unwrap_or(0);
    let scale_label = ui_scale_label(config.ui_scale.clamp(0.5, 2.0));
    let selected_scale = UI_SCALE_OPTIONS
      .iter()
      .position(|scale| ui_scale_label(*scale) == scale_label)
      .unwrap_or(0);

    Vec::from([
      #[cfg(not(target_os = "macos"))]
      self.keybind_group(),
      group(
        "Overlay",
        [
          self.dropdown(
            "Display",
            "The display to show the overlay on",
            display_names,
            selected_display,
            |cfg, idx| cfg.display_idx = Some(idx),
          ),
          self.dropdown(
            "UI Scale",
            "Scale of the overlay elements",
            UI_SCALE_OPTIONS
              .iter()
              .map(|scale| ui_scale_label(*scale))
              .collect(),
            selected_scale,
            |cfg, idx| cfg.ui_scale = UI_SCALE_OPTIONS[idx],
          ),
        ],
      ),
    ])
  }

  fn voice(&self) -> Vec<Element> {
    let config = &self.config;
    let visibility = config
      .display_voice_members
      .clone()
      .unwrap_or_default()
      .to_string();

    vec![
      group(
        "Visibility",
        [self.dropdown(
          "Display Voice Members",
          "Control when and how voice users are visible",
          to_options(VOICE_DISPLAY_OPTIONS),
          option_index(VOICE_DISPLAY_OPTIONS, &visibility),
          |cfg, idx| {
            cfg.display_voice_members = Some(VOICE_DISPLAY_OPTIONS[idx].to_string().into());
          },
        )],
      ),
      group(
        "Position",
        [
          self.alignment(
            "Voice Alignment",
            "Screen position for voice users",
            config.user_alignment.as_deref().unwrap_or("topleft"),
            |cfg, v| cfg.user_alignment = Some(v),
          ),
          self.number(
            "X Offset (px)",
            None,
            config.user_offset_x,
            false,
            |cfg, n| cfg.user_offset_x = n,
          ),
          self.number(
            "Y Offset (px)",
            None,
            config.user_offset_y,
            false,
            |cfg, n| cfg.user_offset_y = n,
          ),
        ],
      ),
    ]
  }

  fn notifications(&self) -> Vec<Element> {
    let config = &self.config;

    let mut behavior = vec![
      self.toggle(
        "Enable Notifications",
        "Show notifications for incoming messages, calls, etc.",
        config.enable_message_notifications,
        |cfg, value| cfg.enable_message_notifications = value,
      ),
      self.toggle(
        "Semi-Transparent Notifications",
        "Fade notifications when the overlay is closed",
        config.messages_semitransparent,
        |cfg, value| cfg.messages_semitransparent = value,
      ),
    ];

    if !self.standalone {
      let app = self.app.clone();
      behavior.push(wide_button("Send Test Notification", move |_| {
        app.notify(Notification {
          title: "Test Notification".into(),
          body: "This is a test notification, triggered manually from the configuration window!"
            .into(),
          icon: "https://avatars.githubusercontent.com/u/25207995?v=4".into(),
          ..Default::default()
        });
      }));
    }

    vec![
      group("Behavior", behavior),
      group(
        "Position",
        [
          self.alignment(
            "Notification Alignment",
            "Screen position for notifications",
            config.message_alignment.as_deref().unwrap_or("topright"),
            |cfg, v| cfg.message_alignment = Some(v),
          ),
          self.number(
            "X Offset (px)",
            None,
            config.message_offset_x,
            false,
            |cfg, n| cfg.message_offset_x = n,
          ),
          self.number(
            "Y Offset (px)",
            None,
            config.message_offset_y,
            false,
            |cfg, n| cfg.message_offset_y = n,
          ),
        ],
      ),
    ]
  }

  fn appearance(&self) -> Vec<Element> {
    let config = &self.config;

    vec![
      group(
        "Theme",
        [
          self.color(
            "Accent Color",
            "The accent color for the overlay",
            config.accent,
            |cfg, v| cfg.accent = to_tuple(v),
          ),
          self.color(
            "Text Color",
            "The text color for the overlay",
            config.text_color,
            |cfg, v| cfg.text_color = to_tuple(v),
          ),
          self.number(
            "Border Radius (px)",
            Some("Corner radius used by panels and buttons"),
            config.border_radius,
            false,
            |cfg, n: f32| cfg.border_radius = n.max(0.),
          ),
        ],
      ),
      group(
        "Templates",
        [
          self.template(
            "User Template",
            "Template used to render voice users.",
            list_user_templates(),
            config.user_template.as_deref(),
            |cfg, v| cfg.user_template = v,
          ),
          self.template(
            "Notification Template",
            "Template used to render notifications.",
            list_notification_templates(),
            config.notification_template.as_deref(),
            |cfg, v| cfg.notification_template = v,
          ),
          wide_button("Open Templates Folder", |_| {
            open_templates_folder();
          }),
        ],
      ),
    ]
  }

  fn connection(&self) -> Vec<Element> {
    let config = &self.config;

    vec![group("Discord", [
      self.dropdown(
        "Connection Mode",
        "Set the communication method between Orbolay and Discord. If using an offical client, use \"ipc\", otherwise, use \"websocket\".",
        to_options(TRANSPORT_MODES),
        option_index(TRANSPORT_MODES, &config.transport_mode.to_string()),
        |cfg, idx| cfg.transport_mode = TRANSPORT_MODES[idx].to_string().into(),
      ),
      self.number(
        "Websocket Port",
        Some("Port the websocket server listens on (websocket mode only). Requires Restart."),
        config.port.unwrap_or(6888),
        config.transport_mode != TransportMode::Websocket,
        |cfg, n: u16| cfg.port = Some(n),
      ),
    ])]
  }

  fn advanced(&self) -> Vec<Element> {
    let config = &self.config;

    vec![group(
      "Rendering",
      [
        self.toggle(
          "Force Software Renderer",
          "Use software rendering instead of hardware acceleration. Requires restart.",
          config.software_rendering.unwrap_or(false),
          |cfg, value| cfg.software_rendering = Some(value),
        ),
        #[cfg(target_os = "linux")]
        self.toggle(
          "Run with XWayland",
          "Create the window under XWayland. Requires restart.",
          config.xwayland,
          |cfg, value| cfg.xwayland = value,
        ),
      ],
    )]
  }
}
