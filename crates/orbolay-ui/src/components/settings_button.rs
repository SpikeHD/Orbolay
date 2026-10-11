use freya::prelude::*;

use crate::{
  config::CornerAlignment,
  util::scale::{GapsScaleExt, UiScale},
};

static SETTINGS_SVG: &[u8] = include_bytes!("../../../../assets/settings.svg");

#[derive(PartialEq)]
pub struct SettingsButton {
  pub on_click: EventHandler<()>,
  pub alignment: CornerAlignment,
  pub ui_scale: f32,
}

impl Component for SettingsButton {
  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    let on_click = self.on_click.clone();

    rect()
      .position(self.alignment.to_position())
      .padding(Gaps::new_all(16.0_f32).scaled(scale.factor()))
      .background(Color::TRANSPARENT)
      .cursor(CursorIcon::Pointer)
      .on_press(move |_| on_click.call(()))
      .child(
        SvgViewer::new(SETTINGS_SVG)
          .width(Size::px(scale.px(32.0)))
          .height(Size::px(scale.px(32.0)))
          .opacity(0.4),
      )
  }
}
