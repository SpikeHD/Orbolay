use freya::prelude::*;
use rdev::Key;

use crate::{
  configurator::{dropdown::DropdownControl, input::InputControl, keybind::KeybindControl, toggle::ToggleControl},
  util::colors::MUTED_GRAY,
};

/// Typed change value emitted by `SettingRow::on_change`.
/// String-based controls (Toggle, Dropdown, Input) emit `Value`; the
/// Keybind control emits `Keys` so callers never have to re-parse strings.
#[derive(PartialEq)]
pub enum SettingChange {
  Value(String),
  Keys(Vec<Key>),
}

#[derive(PartialEq)]
pub struct SettingRow {
  pub name: String,
  pub description: Option<String>,
  pub kind: SettingKind,
  pub on_change: EventHandler<SettingChange>,
}

#[derive(PartialEq)]
pub enum SettingKind {
  Toggle(bool),
  Dropdown(Vec<String>, Option<String>),
  Input(Option<String>),
  Keybind(Option<Vec<Key>>),
}

impl Component for SettingRow {
  fn render(&self) -> impl IntoElement {
    let name = self.name.clone();
    let description = self.description.clone();

    let oc_toggle = self.on_change.clone();
    let oc_dropdown = self.on_change.clone();
    let oc_input = self.on_change.clone();
    let oc_keybind = self.on_change.clone();

    let toggle_initial = match &self.kind {
      SettingKind::Toggle(b) => Some(*b),
      _ => None,
    };
    let dropdown_data = match &self.kind {
      SettingKind::Dropdown(opts, initial) => Some((opts.clone(), initial.clone())),
      _ => None,
    };
    let input_initial = match &self.kind {
      SettingKind::Input(initial) => Some(initial.clone()),
      _ => None,
    };
    let keybind_initial = match &self.kind {
      SettingKind::Keybind(initial) => Some(initial.clone()),
      _ => None,
    };

    rect()
      .direction(Direction::Vertical)
      .width(Size::fill())
      .padding(Gaps::new(10., 12., 10., 12.))
      .child(
        rect()
          .direction(Direction::Horizontal)
          .main_align(Alignment::SpaceBetween)
          .cross_align(Alignment::Center)
          .width(Size::fill())
          .child(label().text(name).color(Color::WHITE).font_size(14.))
          .map(toggle_initial, move |el, initial| {
            el.child(ToggleControl::new(
              initial,
              EventHandler::new(move |v: String| oc_toggle.call(SettingChange::Value(v))),
            ))
          })
          .map(dropdown_data, move |el, (opts, initial)| {
            el.child(DropdownControl::new(
              opts,
              initial,
              EventHandler::new(move |v: String| oc_dropdown.call(SettingChange::Value(v))),
            ))
          })
          .map(input_initial, move |el, initial| {
            el.child(InputControl::new(
              initial,
              EventHandler::new(move |v: String| oc_input.call(SettingChange::Value(v))),
            ))
          })
          .map(keybind_initial, move |el, initial| {
            el.child(KeybindControl::new(
              initial.map(|keys| {
                keys.iter()
                  .map(|k| format!("{:?}", k))
                  .collect::<Vec<String>>()
                  .join(", ")
              }),
              EventHandler::new(move |keys: Vec<Key>| oc_keybind.call(SettingChange::Keys(keys))),
              EventHandler::new(|_| {}),
              EventHandler::new(|_| {}),
            ))
          }),
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
