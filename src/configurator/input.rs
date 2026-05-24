use freya::prelude::*;

#[derive(PartialEq)]
pub struct InputControl {
  initial: Option<String>,
  on_change: EventHandler<String>,
}

impl InputControl {
  pub fn new(initial: Option<String>, on_change: EventHandler<String>) -> Self {
    Self { initial, on_change }
  }
}

impl Component for InputControl {
  fn render(&self) -> impl IntoElement {
    let on_change = self.on_change.clone();
    let value = use_state(|| self.initial.clone().unwrap_or_default());
    Input::new(value).on_submit(move |v| on_change.call(v))
  }
}
