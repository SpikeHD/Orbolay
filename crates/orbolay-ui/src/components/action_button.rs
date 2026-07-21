use freya::prelude::*;

use orbolay_core::payloads::NotificationKind;

use crate::util::{
  scale::{GapsScaleExt, UiScale},
  theme::{self, Theme},
};

#[derive(PartialEq)]
pub struct ActionButton {
  pub func: Callback<(), ()>,
  pub label: String,
  pub kind: NotificationKind,
  pub theme: Theme,
  pub ui_scale: f32,
}

impl Component for ActionButton {
  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    let func = self.func.clone();
    let is_secondary = self.kind == NotificationKind::Secondary;
    let bg = if is_secondary {
      theme::TRANSPARENT
    } else {
      theme::GREEN
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
      .height(Size::px(scale.px(30.0)))
      .corner_radius(CornerRadius::new_all(self.theme.border_radius))
      .margin(Gaps::new(0., 6., 0., 0.).scaled(scale.factor()))
      .padding(Gaps::new_all(4.).scaled(scale.factor()))
      .background(bg)
      .maybe(is_secondary, |el| {
        el.border(Border::new().fill(self.theme.muted_gray).width(1.))
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
          .font_size(scale.px(14.))
          .color(self.theme.text_color)
          .text(self.label.clone()),
      )
  }
}
