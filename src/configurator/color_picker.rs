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
    let hex_value = use_state(|| color_to_hex(self.initial));

    let apply_color = {
      let mut value = value;
      let mut hex_value = hex_value;
      let on_change = on_change.clone();
      move |color: Color| {
        value.set(color);
        hex_value.set(color_to_hex(color));
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
          parse_hex_color(&hex)
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
      .child(
        ContextMenuViewer::new()
      )
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

fn color_to_hex(color: Color) -> String {
  let (r, g, b) = theme::to_tuple(color);
  format!("#{r:02X}{g:02X}{b:02X}")
}

fn parse_hex_color(value: &str) -> Option<Color> {
  let hex = value.trim().trim_start_matches('#');
  if hex.len() != 6 {
    return None;
  }

  let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
  let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
  let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

  Some(Color::from_rgb(r, g, b))
}
