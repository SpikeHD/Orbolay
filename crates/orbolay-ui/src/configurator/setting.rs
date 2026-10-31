use freya::prelude::*;

use crate::util::theme::MUTED_GRAY;

#[derive(PartialEq)]
pub struct SettingRow {
  pub name: String,
  pub description: Option<String>,
  pub control: Element,
  pub disabled: bool,
}

impl Component for SettingRow {
  fn render(&self) -> impl IntoElement {
    let name = self.name.clone();
    let description = self.description.clone();

    rect()
      .direction(Direction::Horizontal)
      .content(Content::Flex)
      .cross_align(Alignment::Center)
      .spacing(16.)
      .width(Size::fill())
      .padding(Gaps::new_symmetric(10., 12.))
      .opacity(if self.disabled { 0.4_f32 } else { 1.0_f32 })
      .child(
        rect()
          .width(Size::flex(1.))
          .child(label().text(name).color(Color::WHITE).font_size(14.))
          .map(description, |el, desc| {
            el.child(
              paragraph()
                .span(Span::new(desc).color(MUTED_GRAY).font_size(12.))
                .width(Size::fill())
                .margin(Gaps::new(4., 0., 0., 0.)),
            )
          }),
      )
      .child(self.control.clone())
  }
}
