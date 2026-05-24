use freya::prelude::*;

use crate::{configurator::{dropdown::DropdownControl, input::InputControl, toggle::ToggleControl}, util::colors::MUTED_GRAY};

#[derive(PartialEq)]
pub struct SettingRow {
  pub name: String,
  pub description: Option<String>,
  pub kind: SettingKind,
  pub on_change: EventHandler<String>,
}

#[derive(PartialEq)]
pub enum SettingKind {
  Toggle(bool),
  Dropdown(Vec<String>, Option<String>),
  Input(Option<String>),
}

impl Component for SettingRow {
  fn render(&self) -> impl IntoElement {
    let name = self.name.clone();
    let description = self.description.clone();

    let oc_toggle = self.on_change.clone();
    let oc_dropdown = self.on_change.clone();
    let oc_input = self.on_change.clone();

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
            el.child(ToggleControl::new(initial, oc_toggle))
          })
          .map(dropdown_data, move |el, (opts, initial)| {
            el.child(DropdownControl::new(opts, initial, oc_dropdown))
          })
          .map(input_initial, move |el, initial| {
            el.child(InputControl::new(initial, oc_input))
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
