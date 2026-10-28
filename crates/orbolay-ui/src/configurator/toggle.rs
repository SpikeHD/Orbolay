use freya::prelude::*;

use crate::configurator::setting::SettingChange;

#[derive(PartialEq, Clone)]
pub struct ToggleControl {
  initial: bool,
  on_change: EventHandler<SettingChange>,
}

impl ToggleControl {
  pub fn new(initial: bool, on_change: EventHandler<SettingChange>) -> Self {
    Self { initial, on_change }
  }
}

impl Component for ToggleControl {
  fn render(&self) -> impl IntoElement {
    let on_change = self.on_change.clone();
    let mut toggled = use_state(|| self.initial);
    let setter = WritableSetter::new(toggled, move |value: bool| {
      toggled.set(value);
      on_change.call(SettingChange::Bool(value));
    });
    Switch::new(setter)
  }
}
