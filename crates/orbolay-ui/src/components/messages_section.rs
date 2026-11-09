use std::str::FromStr;

use freya::prelude::*;

use orbolay_core::{app_state::AppState, payloads::Notification, util::text::estimate_lines};

use crate::{
  components::MessageRow,
  config::{AxisAlignment, CornerAlignment},
  util::notification_template::{BODY_WIDTH_PX, MULTILINE_LINES, load_notification_template},
  util::{scale::UiScale, theme::Theme},
};

#[derive(PartialEq)]
pub struct MessagesSection {
  pub messages: Vec<Notification>,
  pub chat_messages: Vec<Notification>,
  pub is_open: bool,
  pub is_censor: bool,
  pub message_alignment: String,
  pub message_offset_x: i32,
  pub message_offset_y: i32,
  pub messages_semitransparent: bool,
  pub notification_template: Option<String>,
  pub multiline: bool,
  pub app_state: State<AppState>,
  pub theme: Theme,
  pub ui_scale: f32,
}

impl Component for MessagesSection {
  fn render(&self) -> impl IntoElement {
    // unwrap: this does not fail
    let alignment = CornerAlignment::from_str(&self.message_alignment).unwrap();
    let scale = UiScale::new(self.ui_scale);
    let gaps = alignment.to_gaps(
      scale.int(self.message_offset_x),
      scale.int(self.message_offset_y),
    );
    let opacity = if self.messages_semitransparent && !self.is_open {
      0.5_f32
    } else {
      1.0_f32
    };

    let builtin = load_notification_template(self.notification_template.as_deref()).is_builtin();

    let rows: Vec<&Notification> = if alignment.y == AxisAlignment::End {
      self.chat_messages.iter().chain(&self.messages).collect()
    } else {
      self.messages.iter().chain(&self.chat_messages).collect()
    };

    rows.into_iter().fold(
      rect()
        .direction(Direction::Vertical)
        .cross_align(alignment.x.to_freya())
        .main_align(alignment.y.to_freya())
        .position(Position::new_absolute().top(0.).left(0.))
        .background(Color::TRANSPARENT)
        .height(Size::fill())
        .width(Size::fill())
        .padding(gaps)
        .opacity(opacity),
      |el, message| {
        if self.is_censor {
          el
        } else {
          let has_actions =
            message.actions.is_some() && !message.actions.as_ref().unwrap().is_empty();

          el.child(MessageRow {
            app_state: self.app_state,
            message: message.clone(),
            theme: self.theme,
            box_size: (
              400,
              row_height(message, has_actions, self.multiline && builtin),
            ),
            notification_template: self.notification_template.clone(),
            multiline: self.multiline,
            ui_scale: scale.factor(),
          })
        }
      },
    )
  }
}

fn row_height(message: &Notification, has_actions: bool, multiline: bool) -> u32 {
  let base = if has_actions { 100 } else { 66 };
  if !multiline {
    return base;
  }

  let lines = estimate_lines(&message.body, BODY_WIDTH_PX, MULTILINE_LINES);
  base + (lines - 1) * 18
}
