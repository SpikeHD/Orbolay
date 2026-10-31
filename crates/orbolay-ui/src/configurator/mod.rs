use std::cell::Cell;
use std::sync::atomic::Ordering;

use display_info::DisplayInfo;
use freya::{prelude::*, winit::window::WindowId};

use orbolay_core::{
  app_state::{AppHandle, SharedAppState},
  config::{Config, load_config},
};

use crate::util::theme::{GRAY, MUTED_GRAY};

use sections::{Section, Settings, apply_config};
use sidebar::sidebar;

mod buttons;
mod color_picker;
mod commit;
mod dropdown;
mod input;
#[cfg(not(target_os = "macos"))]
mod keybind;
mod sections;
mod setting;
mod sidebar;
mod toggle;

const WIDTH: f32 = 760.;
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

const UI_SCALE_OPTIONS: &[f32] = &[0.5, 0.75, 1.0, 1.25, 1.5, 1.75, 2.0];

fn ui_scale_label(scale: f32) -> String {
  format!("{:.0}%", scale * 100.0)
}

fn parse_ui_scale_label(value: &str) -> Option<f32> {
  let percent = value.strip_suffix('%')?.trim().parse::<f32>().ok()?;
  Some(percent / 100.0)
}

#[derive(Clone, Copy)]
enum ConfiguratorWindow {
  Opening,
  Open(WindowId),
}

thread_local! {
  static CONFIGURATOR: Cell<Option<ConfiguratorWindow>> = const { Cell::new(None) };
}

pub fn open_configurator(app: &AppHandle) {
  match CONFIGURATOR.get() {
    Some(ConfiguratorWindow::Open(id)) => show_window(id),
    Some(ConfiguratorWindow::Opening) => {}
    None => {
      CONFIGURATOR.set(Some(ConfiguratorWindow::Opening));

      let window = configurator_window(app.clone(), false).with_on_close(|_, _| {
        CONFIGURATOR.set(None);
        CloseDecision::Close
      });

      spawn_global(async move {
        let id = Platform::get().launch_window(window).await;

        if let Some(ConfiguratorWindow::Opening) = CONFIGURATOR.get() {
          CONFIGURATOR.set(Some(ConfiguratorWindow::Open(id)));
          show_window(id);
        }
      });
    }
  }
}

fn show_window(id: WindowId) {
  Platform::get().with_window(id, |window| {
    window.set_minimized(false);
    window.focus_window();
  });
}

pub fn open_configurator_standalone() {
  // Basically a blocking, standalone version of open_configurator
  let shared = SharedAppState::default();
  let (redraw_tx, _) = flume::unbounded();
  let (open_config_tx, _) = flume::unbounded();
  let app = AppHandle::new(shared, redraw_tx, open_config_tx);

  if let Some(saved) = load_config() {
    app.update(|state| state.config = saved);
  }

  launch(LaunchConfig::new().with_window(configurator_window(app, true)));
}

fn configurator_window(app: AppHandle, standalone: bool) -> WindowConfig {
  WindowConfig::new(move || configurator(app.clone(), standalone))
    .with_background(GRAY)
    .with_size(WIDTH as f64, HEIGHT as f64)
    .with_min_size(WIDTH as f64, 400.)
    .with_title("Orbolay Configurator")
}

fn configurator(app: AppHandle, standalone: bool) -> impl IntoElement {
  use_init_theme(dark_theme);

  // Make the recording flag available to KeybindControl
  let recording_flag = app.read(|state| state.recording_keybind.clone());
  let recording = recording_flag.clone();
  use_provide_context(move || recording_flag);

  let local_config = use_state(|| app.read(|state| state.config.clone()));
  let mut reset_version = use_state(|| 0usize);
  let section = use_state(|| Section::General);
  let window_id = use_hook(Platform::window_id);

  let close = EventHandler::new(move |()| Platform::get().close_window(window_id));
  let close_shortcut = close.clone();

  let on_reset = EventHandler::new({
    let app = app.clone();
    move |()| {
      apply_config(&app, local_config, |cfg| *cfg = Config::default());
      reset_version.set(reset_version() + 1);
    }
  });

  let current = section();
  let settings = Settings {
    app,
    local_config,
    config: local_config.read().clone(),
    standalone,
  };

  let page = rect()
    .width(Size::fill())
    .padding(Gaps::new(24., 24., 4., 24.))
    .child(
      label()
        .text(current.title())
        .color(Color::WHITE)
        .font_size(22.)
        .font_weight(FontWeight::BOLD),
    )
    .child(
      label()
        .text(current.description())
        .color(MUTED_GRAY)
        .font_size(13.)
        .margin(Gaps::new(2., 0., 20., 0.)),
    )
    .children(settings.groups(current));

  rect()
    .width(Size::fill())
    .height(Size::fill())
    .background(GRAY)
    .direction(Direction::Horizontal)
    .content(Content::Flex)
    .on_global_key_down(move |e: Event<KeyboardEventData>| {
      let is_close = e.modifiers.contains(Modifiers::ctrl_or_meta()) && e.code == Code::KeyW;
      if is_close && !recording.load(Ordering::Relaxed) {
        close_shortcut.call(());
      }
    })
    .child(sidebar(section, on_reset, close))
    .child(
      ScrollView::new()
        .key((current, reset_version()))
        .width(Size::flex(1.))
        .child(page),
    )
}
