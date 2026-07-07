use freya::prelude::*;

#[cfg(not(target_os = "macos"))]
use rdev::Key;

use crate::{
  configurator::{
    color_picker::ColorPickerControl, dropdown::DropdownControl, input::InputControl,
    toggle::ToggleControl,
  },
  util::theme::MUTED_GRAY,
};

#[cfg(not(target_os = "macos"))]
use crate::configurator::keybind::KeybindControl;

#[derive(PartialEq)]
pub enum SettingChange {
  Value(String),
  Bool(bool),
  Color(Color),
  #[cfg(not(target_os = "macos"))]
  Keys(Vec<Key>),
}

#[derive(PartialEq)]
pub struct SettingRow {
  pub name: String,
  pub description: Option<String>,
  pub kind: SettingKind,
  pub on_change: EventHandler<SettingChange>,
  pub disabled: bool,
}

#[derive(PartialEq, Clone)]
pub enum SettingKind {
  Toggle(bool),
  Dropdown(Vec<String>, Option<String>),
  Input(Option<String>),
  Color(Color),
  #[cfg(not(target_os = "macos"))]
  Keybind(Option<Vec<Key>>),
}

impl SettingKind {
  fn element(self, on_change: EventHandler<SettingChange>) -> Element {
    match self {
      SettingKind::Dropdown(options, initial) => {
        DropdownControl::new(options, initial, on_change.clone()).into()
      }
      SettingKind::Input(initial) => InputControl::new(initial, on_change.clone()).into(),
      SettingKind::Toggle(initial) => ToggleControl::new(initial, on_change.clone()).into(),
      SettingKind::Color(initial) => ColorPickerControl::new(initial, on_change.clone()).into(),
      #[cfg(not(target_os = "macos"))]
      SettingKind::Keybind(initial) => KeybindControl::new(initial, on_change.clone()).into(),
    }
  }
}

impl Component for SettingRow {
  fn render(&self) -> impl IntoElement {
    let name = self.name.clone();
    let description = self.description.clone();

    rect()
      .direction(Direction::Vertical)
      .width(Size::fill())
      .padding(Gaps::new(10., 12., 10., 12.))
      .opacity(if self.disabled { 0.4 } else { 1.0 })
      .child(
        rect()
          .direction(Direction::Horizontal)
          .main_align(Alignment::SpaceBetween)
          .cross_align(Alignment::Center)
          .width(Size::fill())
          .child(label().text(name).color(Color::WHITE).font_size(14.))
          .child(self.kind.clone().element(self.on_change.clone())),
      )
      .map(description, |el, desc| {
        el.child(
          label()
            .text(desc)
            .color(MUTED_GRAY)
            .font_size(12.)
            .margin(Gaps::new(4., 0., 0., 0.)),
        )
      })
  }
}
