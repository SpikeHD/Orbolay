use freya::prelude::*;

use crate::configurator::setting::SettingChange;

#[derive(PartialEq, Clone)]
pub struct ColorPickerControl {
  initial: Color,
  on_change: EventHandler<SettingChange>,
}

impl ColorPickerControl {
  pub fn new(initial: Color, on_change: EventHandler<SettingChange>) -> Self {
    Self { initial, on_change }
  }
}

impl Component for ColorPickerControl {
  fn render(&self) -> impl IntoElement {
    let on_change = self.on_change.clone();
    let value = use_state(|| self.initial);
    let on_change = {
      let mut value = value;
      move |color: Color| {
        value.set(color);
        on_change.call(SettingChange::Color(*value.read()));
      }
    };

    ColorPicker::new(on_change).value(*value.read())
  }
}
