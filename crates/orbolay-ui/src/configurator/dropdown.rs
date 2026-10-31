use freya::prelude::*;

#[derive(PartialEq, Clone)]
pub struct DropdownControl {
  options: Vec<String>,
  initial: usize,
  on_change: EventHandler<usize>,
}

impl DropdownControl {
  pub fn new(options: Vec<String>, initial: usize, on_change: EventHandler<usize>) -> Self {
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
    let mut selected = use_state(|| self.initial);

    let selected_idx = selected();
    let selected_label = options.get(selected_idx).cloned().unwrap_or_default();

    Select::new()
      .selected_item(selected_label)
      .children(options.into_iter().enumerate().map(|(i, val)| -> Element {
        let on_change = on_change.clone();
        MenuItem::new()
          .selected(selected_idx == i)
          .on_press(move |_| {
            selected.set(i);
            on_change.call(i);
          })
          .child(val)
          .into()
      }))
  }
}
