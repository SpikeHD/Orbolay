use freya::prelude::*;

use crate::{payloads::NotificationKind, util::colors};

#[derive(PartialEq)]
pub struct ActionButton {
  pub func: Callback<(), ()>,
  pub label: String,
  pub kind: NotificationKind,
}

impl Component for ActionButton {
  fn render(&self) -> impl IntoElement {
    let func = self.func.clone();
    let is_secondary = self.kind == NotificationKind::Secondary;
    let (bg, text_color) = if is_secondary {
      (colors::TRANSPARENT, Color::WHITE)
    } else {
      (colors::GREEN, Color::WHITE)
    };

    rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Center)
      .cross_align(Alignment::Center)
      .height(Size::px(30.))
      .width(Size::px(80.))
      .corner_radius(CornerRadius::new_all(5.))
      .background(bg)
      .maybe(is_secondary, |el| {
        el.border(Border::new().fill(colors::MUTED_GRAY).width(1.))
      })
      .on_press(move |_| {
        func.call(());
      })
      .child(
        label()
          .font_size(14.)
          .color(text_color)
          .text(self.label.clone()),
      )
  }
}
