use std::cell::Cell;
use std::sync::atomic::Ordering;

use display_info::DisplayInfo;
use freya::{prelude::*, winit::window::WindowId};

use orbolay_core::{
  app_state::{AppHandle, SharedAppState},
  config::{Config, load_config, save_config},
};

use crate::util::theme::{DARKISH_GRAY, GRAY, LIGHT_GRAY, MUTED_GRAY, RED, SUPERLIGHT_GRAY};

use sections::{Section, Settings};

mod color_picker;
mod commit;
mod dropdown;
mod input;
#[cfg(not(target_os = "macos"))]
mod keybind;
mod sections;
mod setting;
mod toggle;

const WIDTH: f32 = 760.;
const SIDEBAR_WIDTH: f32 = 180.;
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

fn action_button(
  text: &'static str,
  background: Color,
  hover_background: Color,
  color: Color,
  on_press: impl Into<EventHandler<Event<PressEventData>>>,
) -> impl IntoElement {
  Button::new()
    .width(Size::fill())
    .height(Size::px(32.0_f32))
    .corner_radius(8.)
    .background(background)
    .hover_background(hover_background)
    .border_fill(Color::TRANSPARENT)
    .cursor(CursorIcon::Pointer)
    .on_press(on_press)
    .child(label().text(text).color(color).font_size(13.))
}

#[derive(PartialEq)]
struct ResetButton {
  on_reset: EventHandler<()>,
}

impl Component for ResetButton {
  fn render(&self) -> impl IntoElement {
    let on_reset = self.on_reset.clone();
    let mut armed = use_state(|| false);

    let (text, background, hover_background, color) = if armed() {
      ("Confirm reset", RED, RED, Color::WHITE)
    } else {
      ("Reset to Defaults", DARKISH_GRAY, LIGHT_GRAY, RED)
    };

    action_button(text, background, hover_background, color, move |_| {
      if armed() {
        on_reset.call(());
      }
      armed.set(!armed());
    })
  }
}

fn nav_item(section: Section, mut current: State<Section>) -> Element {
  let active = section == current();

  Activable::new(
    SideBarItem::new()
      .background(DARKISH_GRAY)
      .hover_background(GRAY)
      .active_background(LIGHT_GRAY)
      .corner_radius(8.)
      .on_press(move |_| current.set(section))
      .child(
        label()
          .text(section.title())
          .color(if active {
            Color::WHITE
          } else {
            SUPERLIGHT_GRAY
          })
          .font_size(14.),
      ),
  )
  .active(active)
  .into()
}

fn sidebar(
  current: State<Section>,
  on_reset: EventHandler<()>,
  on_close: EventHandler<()>,
) -> impl IntoElement {
  rect()
    .width(Size::px(SIDEBAR_WIDTH))
    .height(Size::fill())
    .padding(12.)
    .background(DARKISH_GRAY)
    .content(Content::Flex)
    .child(
      label()
        .text("Orbolay")
        .color(Color::WHITE)
        .font_size(18.)
        .font_weight(FontWeight::BOLD)
        .margin(Gaps::new(6., 0., 16., 12.)),
    )
    .child(
      rect()
        .width(Size::fill())
        .height(Size::flex(1.))
        .spacing(2.)
        .children(
          Section::ALL
            .into_iter()
            .map(|section| nav_item(section, current)),
        ),
    )
    .child(
      label()
        .text("Press \"C\" with the overlay open to open this window again!")
        .color(MUTED_GRAY)
        .font_size(11.)
        .width(Size::fill())
        .margin(Gaps::new(0., 4., 8., 4.)),
    )
    .child(
      rect()
        .width(Size::fill())
        .spacing(4.)
        .child(ResetButton { on_reset })
        .child(action_button(
          "Close",
          LIGHT_GRAY,
          MUTED_GRAY,
          Color::WHITE,
          move |_| on_close.call(()),
        )),
    )
}

fn configurator(app: AppHandle, standalone: bool) -> impl IntoElement {
  use_init_theme(dark_theme);

  // Make the recording flag available to KeybindControl
  let recording_flag = app.read(|state| state.recording_keybind.clone());
  let recording = recording_flag.clone();
  use_provide_context(move || recording_flag);

  let mut local_config = use_state(|| app.read(|state| state.config.clone()));
  let mut reset_version = use_state(|| 0usize);
  let section = use_state(|| Section::General);
  let window_id = use_hook(Platform::window_id);

  let close = EventHandler::new(move |()| Platform::get().close_window(window_id));
  let close_shortcut = close.clone();

  let on_reset = EventHandler::new({
    let app = app.clone();
    move |()| {
      let updated = app.update(|state| {
        state.config = Config::default();
        state.config.clone()
      });
      save_config(&updated);
      local_config.set(updated);
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
    .key(reset_version())
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
      rect().width(Size::flex(1.)).height(Size::fill()).child(
        ScrollView::new()
          .key(current)
          .height(Size::fill())
          .width(Size::fill())
          .direction(Direction::Vertical)
          .child(page),
      ),
    )
}
