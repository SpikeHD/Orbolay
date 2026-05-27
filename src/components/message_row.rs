use freya::prelude::*;

use crate::{
  app_state::AppState,
  payloads::MessageNotification,
  util::{bridge::BridgeMessage, colors, image::avatar_image, text::strip},
};

#[derive(PartialEq)]
pub struct MessageRow {
  pub app_state: State<AppState>,
  pub message: MessageNotification,
}

impl Component for MessageRow {
  fn render(&self) -> impl IntoElement {
    let mut app_state = self.app_state;
    let message = self.message.clone();

    rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Start)
      .cross_align(Alignment::Center)
      .height(Size::px(70.))
      .max_width(Size::px(400.))
      .margin(Gaps::new_all(6.))
      .corner_radius(CornerRadius::new_all(10.))
      .background(colors::GRAY)
      .overflow(Overflow::Clip)
      .on_press(move |_| {
        app_state.write().send(BridgeMessage {
          cmd: "NAVIGATE".to_string(),
          data: serde_json::json!({
            "guild_id": message.guild_id,
            "channel_id": message.channel_id,
            "message_id": message.message_id,
          }),
        })
      })
      .child(
        rect()
          .direction(Direction::Horizontal)
          .main_align(Alignment::Start)
          .cross_align(Alignment::Center)
          .width(Size::fill())
          .height(Size::fill())
          .child(
            avatar_image(&self.message.icon, None)
              .width(Size::px(54.))
              .height(Size::px(54.))
              .margin(Gaps::new(0., 0., 0., 10.)),
          )
          .child(
            rect()
              .direction(Direction::Vertical)
              .main_align(Alignment::Start)
              .cross_align(Alignment::Start)
              .height(Size::fill())
              .width(Size::fill())
              .margin(Gaps::new(6., 10., 6., 6.))
              .child(
                label()
                  .font_size(14.)
                  .font_weight(FontWeight::BOLD)
                  .color(Color::WHITE)
                  .margin(Gaps::new(0., 0., 4., 0.))
                  .max_lines(1)
                  .text(self.message.title.clone())
                  .text_overflow(TextOverflow::Ellipsis),
              )
              .child(
                label()
                  .font_size(14.)
                  .color(colors::SUPERLIGHT_GRAY)
                  .max_lines(2)
                  .text(strip(&self.message.body))
                  .text_overflow(TextOverflow::Ellipsis),
              ),
          ),
      )
  }
}
