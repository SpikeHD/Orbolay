use freya::prelude::*;

use crate::util::theme::{DARKISH_GRAY, GRAY, LIGHT_GRAY, SUPERLIGHT_GRAY};

use super::buttons::{ResetButton, secondary_button};
use super::sections::Section;

const SIDEBAR_WIDTH: f32 = 180.;

fn nav_item(section: Section, mut current: State<Section>) -> Element {
  let active = section == current();

  Activable::new(
    SideBarItem::new()
      .background(DARKISH_GRAY)
      .hover_background(GRAY)
      .active_background(LIGHT_GRAY)
      .corner_radius(8.)
      .on_press(move |_| current.set(section))
      .child(
        label()
          .text(section.title())
          .color(if active {
            Color::WHITE
          } else {
            SUPERLIGHT_GRAY
          })
          .font_size(14.),
      ),
  )
  .active(active)
  .into()
}

pub fn sidebar(
  current: State<Section>,
  on_reset: EventHandler<()>,
  on_close: EventHandler<()>,
) -> impl IntoElement {
  rect()
    .width(Size::px(SIDEBAR_WIDTH))
    .height(Size::fill())
    .padding(12.)
    .background(DARKISH_GRAY)
    .content(Content::Flex)
    .child(
      label()
        .text("Orbolay")
        .color(Color::WHITE)
        .font_size(18.)
        .font_weight(FontWeight::BOLD)
        .margin(Gaps::new(6., 0., 16., 12.)),
    )
    .child(
      rect()
        .width(Size::fill())
        .height(Size::flex(1.))
        .spacing(2.)
        .children(
          Section::ALL
            .into_iter()
            .map(|section| nav_item(section, current)),
        ),
    )
    .child(
      rect()
        .width(Size::fill())
        .spacing(4.)
        .child(ResetButton { on_reset })
        .child(secondary_button("Close", move |_| on_close.call(()))),
    )
}
