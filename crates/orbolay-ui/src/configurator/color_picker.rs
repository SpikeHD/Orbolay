use freya::prelude::*;

use crate::{configurator::commit::use_commit_on_blur, util::theme};

#[derive(PartialEq, Clone)]
pub struct ColorPickerControl {
  initial: Color,
  on_change: EventHandler<Color>,
}

impl ColorPickerControl {
  pub fn new(initial: Color, on_change: EventHandler<Color>) -> Self {
    Self { initial, on_change }
  }
}

impl Component for ColorPickerControl {
  fn render(&self) -> impl IntoElement {
    let input_id = use_a11y();
    let on_change = self.on_change.clone();
    let mut value = use_state(|| self.initial);
    let mut hex_value = use_state(|| theme::to_hex(self.initial));

    let apply_color = move |color: Color| {
      value.set(color);
      hex_value.set(theme::to_hex(color));
      on_change.call(color);
    };

    use_commit_on_blur(input_id, hex_value, {
      let mut apply_color = apply_color.clone();
      move |hex| {
        if let Some(color) = theme::from_hex(&hex)
          && color != *value.peek()
        {
          apply_color(color);
        }
      }
    });

    rect()
      .direction(Direction::Horizontal)
      .cross_align(Alignment::Center)
      .child(ContextMenuViewer::new())
      .child(
        rect()
          .border(Border::new().fill(theme::MUTED_GRAY).width(1.))
          .corner_radius(5.)
          .child(
            rect()
              .margin(Gaps::new_all(1.))
              .child(ColorPicker::new(apply_color).value(*value.read())),
          ),
      )
      .child(
        rect()
          .padding(Gaps::new(0., 0., 0., 10.))
          .child(Input::new(hex_value).a11y_id(input_id)),
      )
  }
}
