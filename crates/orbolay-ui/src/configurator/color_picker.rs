use freya::prelude::*;

use crate::{configurator::setting::SettingChange, util::theme};

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
    let input_id = use_a11y();
    let input_focus = use_focus(input_id);
    let on_change = self.on_change.clone();
    let value = use_state(|| self.initial);
    let hex_value = use_state(|| theme::to_hex(self.initial));

    let apply_color = {
      let mut value = value;
      let mut hex_value = hex_value;
      let on_change = on_change.clone();
      move |color: Color| {
        value.set(color);
        hex_value.set(theme::to_hex(color));
        on_change.call(SettingChange::Color(color));
      }
    };

    use_side_effect({
      let mut apply_color = apply_color.clone();
      move || {
        if input_focus.read().is_focused() {
          return;
        }

        let parsed_color = {
          let hex = hex_value.read().clone();
          theme::from_hex(&hex)
        };
        let current_color = *value.read();

        if let Some(color) = parsed_color
          && color != current_color
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
