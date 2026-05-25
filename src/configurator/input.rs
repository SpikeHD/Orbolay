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
    let id = use_a11y();
    let focus_status = use_focus(id);
    let on_change = self.on_change.clone();
    let value = use_state(|| self.initial.clone().unwrap_or_default());

    use_side_effect(move || {
      if !focus_status.read().is_focused() {
        on_change.call(value.read().clone());
      }
    });

    Input::new(value).a11y_id(id)
  }
}
