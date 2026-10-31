use freya::prelude::*;

use crate::configurator::commit::use_commit_on_blur;

#[derive(PartialEq, Clone)]
pub struct InputControl {
  initial: String,
  on_change: EventHandler<String>,
  enabled: bool,
}

impl InputControl {
  pub fn new(initial: String, on_change: EventHandler<String>, enabled: bool) -> Self {
    Self {
      initial,
      on_change,
      enabled,
    }
  }
}

impl Component for InputControl {
  fn render(&self) -> impl IntoElement {
    let id = use_a11y();
    let on_change = self.on_change.clone();
    let value = use_state(|| self.initial.clone());

    use_commit_on_blur(id, value, move |text| on_change.call(text));

    Input::new(value).a11y_id(id).enabled(self.enabled)
  }
}
