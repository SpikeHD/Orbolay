use freya::prelude::*;

use crate::configurator::setting::SettingChange;

#[derive(PartialEq, Clone)]
pub struct DropdownControl {
  options: Vec<String>,
  initial: Option<String>,
  on_change: EventHandler<SettingChange>,
}

impl DropdownControl {
  pub fn new(
    options: Vec<String>,
    initial: Option<String>,
    on_change: EventHandler<SettingChange>,
  ) -> Self {
    Self {
      options,
      initial,
      on_change,
    }
  }
}

impl Component for DropdownControl {
  fn render(&self) -> impl IntoElement {
    let options = self.options.clone();
    let on_change = self.on_change.clone();
    let initial_idx = self
      .initial
      .as_ref()
      .and_then(|v| options.iter().position(|o| o == v))
      .unwrap_or(0);
    let mut selected = use_state(move || initial_idx);

    let selected_idx = selected();
    let selected_label = options.get(selected_idx).cloned().unwrap_or_default();

    Select::new()
      .selected_item(selected_label)
      .children(options.iter().enumerate().map(|(i, val)| {
        let val = val.clone();
        let label = val.clone();
        let on_change = on_change.clone();
        MenuItem::new()
          .selected(selected_idx == i)
          .on_press(move |_| {
            selected.set(i);
            on_change.call(SettingChange::Value(val.clone()));
          })
          .child(label)
          .into()
      }))
  }
}
