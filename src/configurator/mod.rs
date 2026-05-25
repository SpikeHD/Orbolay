use freya::prelude::*;

use crate::{
  app_state::SharedAppState,
  config::{Config, save_config},
  util::colors::{GRAY, MUTED_GRAY, TRANSPARENT},
};

use setting::{SettingKind, SettingRow};

mod dropdown;
mod input;
mod setting;
mod toggle;

const WIDTH: f32 = 500.;
const HEIGHT: f32 = 600.;

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

pub fn open_configurator(shared: SharedAppState, redraw_tx: flume::Sender<()>) {
  spawn(async move {
    let _ = Platform::get()
      .launch_window(
        configurator_window(shared, redraw_tx),
      )
      .await;
  });
}

pub fn open_configurator_standalone() {
  // Basically a blocking, standalone version of open_configurator
  let shared = SharedAppState::default();
  let (redraw_tx, _) = flume::unbounded();

  launch(
    LaunchConfig::new().with_window(
      configurator_window(shared.clone(), redraw_tx)
    )
  );
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
  update_fn: impl Fn(&mut Config, String) + 'static,
) -> EventHandler<String> {
  EventHandler::new(move |value: String| {
    let updated = {
      let mut state = shared.write().unwrap();
      update_fn(&mut state.config, value);
      state.config.clone()
    };
    save_config(&updated);
    redraw_tx.send(()).ok();
  })
}

fn configurator(shared: SharedAppState, redraw_tx: flume::Sender<()>) -> impl IntoElement {
  use_init_theme(dark_theme);

  let config = shared.read().unwrap().config.clone();

  rect()
    .width(Size::px(WIDTH))
    .height(Size::px(HEIGHT))
    .background(GRAY)
    .direction(Direction::Vertical)
    .child(
      ScrollView::new()
        .height(Size::fill())
        .width(Size::fill())
        .direction(Direction::Vertical)
        .child(
          rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .padding(Gaps::new_symmetric(0., 16.))
            .child(SettingRow {
              name: "Semi-Transparent Voice Users".into(),
              description: Some(
                "Fade voice users when not actively speaking and the overlay is closed".into(),
              ),
              kind: SettingKind::Toggle(config.voice_semitransparent.unwrap_or(true)),
              on_change: make_updater(shared.clone(), redraw_tx.clone(), |cfg, v| {
                cfg.voice_semitransparent = Some(v == "true");
              }),
            })
            .child(divider())
            .child(SettingRow {
              name: "Semi-Transparent Notifications".into(),
              description: Some("Fade notifications when the overlay is closed".into()),
              kind: SettingKind::Toggle(config.messages_semitransparent),
              on_change: make_updater(shared.clone(), redraw_tx.clone(), |cfg, v| {
                cfg.messages_semitransparent = v == "true";
              }),
            })
            .child(divider())
            .child(SettingRow {
              name: "Enable Keybind".into(),
              description: Some("Toggle overlay visibility with a keybind".into()),
              kind: SettingKind::Toggle(config.is_keybind_enabled.unwrap_or(true)),
              on_change: make_updater(shared.clone(), redraw_tx.clone(), |cfg, v| {
                cfg.is_keybind_enabled = Some(v == "true");
              }),
            })
            .child(divider())
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
              on_change: make_updater(shared.clone(), redraw_tx.clone(), |cfg, v| {
                cfg.user_alignment = Some(v);
              }),
            })
            .child(divider())
            .child(SettingRow {
              name: "Voice X Offset (px)".into(),
              description: None,
              kind: SettingKind::Input(Some(config.user_offset_x.to_string())),
              on_change: make_updater(shared.clone(), redraw_tx.clone(), |cfg, v| {
                if let Ok(n) = v.trim().parse::<i32>() {
                  cfg.user_offset_x = n;
                }
              }),
            })
            .child(divider())
            .child(SettingRow {
              name: "Voice Y Offset (px)".into(),
              description: None,
              kind: SettingKind::Input(Some(config.user_offset_y.to_string())),
              on_change: make_updater(shared.clone(), redraw_tx.clone(), |cfg, v| {
                if let Ok(n) = v.trim().parse::<i32>() {
                  cfg.user_offset_y = n;
                }
              }),
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
              on_change: make_updater(shared.clone(), redraw_tx.clone(), |cfg, v| {
                cfg.message_alignment = Some(v);
              }),
            })
            .child(divider())
            .child(SettingRow {
              name: "Messages X Offset (px)".into(),
              description: None,
              kind: SettingKind::Input(Some(config.message_offset_x.to_string())),
              on_change: make_updater(shared.clone(), redraw_tx.clone(), |cfg, v| {
                if let Ok(n) = v.trim().parse::<i32>() {
                  cfg.message_offset_x = n;
                }
              }),
            })
            .child(divider())
            .child(SettingRow {
              name: "Messages Y Offset (px)".into(),
              description: None,
              kind: SettingKind::Input(Some(config.message_offset_y.to_string())),
              on_change: make_updater(shared.clone(), redraw_tx.clone(), |cfg, v| {
                if let Ok(n) = v.trim().parse::<i32>() {
                  cfg.message_offset_y = n;
                }
              }),
            })
            .child(
              label()
                .text("Press \"C\" with the overlay open to open this window again!")
                .color(MUTED_GRAY)
                .font_size(12.)
                .padding(16.)
                .text_align(TextAlign::Center)
                .width(Size::fill()),
            ),
        ),
    )
}

fn divider() -> impl IntoElement {
  rect()
    .width(Size::fill())
    .height(Size::px(1.))
    .padding(16.)
    .background(TRANSPARENT)
}
