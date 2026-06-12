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
    let mut hovered = use_state(|| false);

    use_drop(move || {
      if *hovered.read() {
        Cursor::set(CursorIcon::default());
      }
    });

    rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Center)
      .cross_align(Alignment::Center)
      .height(Size::px(30.))
      .width(Size::px(80.))
      .corner_radius(CornerRadius::new_all(5.))
      .margin(Gaps::new(0., 6., 0., 0.))
      .background(bg)
      .maybe(is_secondary, |el| {
        el.border(Border::new().fill(colors::MUTED_GRAY).width(1.))
      })
      .on_press(move |_| {
        func.call(());
      })
      .on_pointer_enter(move |_| {
        *hovered.write() = true;
        Cursor::set(CursorIcon::Pointer);
      })
      .on_pointer_leave(move |_| {
        *hovered.write() = false;
        Cursor::set(CursorIcon::default());
      })
      .child(
        label()
          .font_size(14.)
          .color(text_color)
          .text(self.label.clone()),
      )
  }
}
