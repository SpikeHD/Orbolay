use freya::prelude::*;

use crate::{
  app_state::AppState,
  components::ActionButton,
  payloads::Notification,
  util::{bridge::BridgeMessage, colors, image::avatar_image, text::strip},
};

#[derive(PartialEq)]
pub struct MessageRow {
  pub app_state: State<AppState>,
  pub message: Notification,
}

impl Component for MessageRow {
  fn render(&self) -> impl IntoElement {
    let mut app_state = self.app_state;
    let message = self.message.clone();
    let mut hovered = use_state(|| false);

    use_drop(move || {
      if *hovered.read() {
        Cursor::set(CursorIcon::default());
      }
    });

    rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Start)
      .cross_align(Alignment::Start)
      .max_width(Size::px(400.))
      .margin(Gaps::new_all(6.))
      .padding(Gaps::new_all(10.))
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
      .on_pointer_enter(move |_| {
        *hovered.write() = true;
        Cursor::set(CursorIcon::Pointer);
      })
      .on_pointer_leave(move |_| {
        *hovered.write() = false;
        Cursor::set(CursorIcon::default());
      })
      .child(
        avatar_image(&self.message.icon, None)
          .width(Size::px(42.))
          .height(Size::px(42.))
          .margin(Gaps::new(0., 10., 0., 0.)),
      )
      .child(
        rect()
          .direction(Direction::Vertical)
          .main_align(Alignment::Start)
          .cross_align(Alignment::Start)
          .width(Size::fill())
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
          )
          .maybe(self.message.actions.is_some(), |el| {
            el.child(
              rect()
                .direction(Direction::Horizontal)
                .main_align(Alignment::Start)
                .cross_align(Alignment::Center)
                .content(Content::wrap())
                .margin(Gaps::new(6., 0., 0., 0.))
                .children(
                  self
                    .message
                    .actions
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|action| {
                      let func = action.action.clone();
                      ActionButton {
                        func: Callback::new(move |_| {
                          func();
                        }),
                        label: action.label.clone(),
                        kind: action.kind.clone(),
                      }
                      .into()
                    })
                    .collect::<Vec<_>>(),
                ),
            )
          }),
      )
  }
}
