use freya::prelude::*;

#[derive(PartialEq)]
pub struct ToggleControl {
  initial: bool,
  on_change: EventHandler<String>,
}

impl ToggleControl {
  pub fn new(initial: bool, on_change: EventHandler<String>) -> Self {
    Self { initial, on_change }
  }
}

impl Component for ToggleControl {
  fn render(&self) -> impl IntoElement {
    let on_change = self.on_change.clone();
    let mut toggled = use_state(|| self.initial);
    Switch::new().toggled(toggled()).on_toggle(move |_| {
      toggled.toggle();
      on_change.call(if toggled() { "true" } else { "false" }.to_string());
    })
  }
}
