use std::str::FromStr;

use freya::prelude::*;

use orbolay_core::{app_state::AppState, payloads::Notification};

use crate::{
  components::MessageRow,
  config::CornerAlignment,
  util::{scale::UiScale, theme::Theme},
};

#[derive(PartialEq)]
pub struct MessagesSection {
  pub messages: Vec<Notification>,
  pub is_open: bool,
  pub is_censor: bool,
  pub message_alignment: String,
  pub message_offset_x: i32,
  pub message_offset_y: i32,
  pub messages_semitransparent: bool,
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

    self.messages.iter().fold(
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
          el.child(MessageRow {
            app_state: self.app_state,
            message: message.clone(),
            theme: self.theme,
            ui_scale: scale.factor(),
          })
        }
      },
    )
  }
}
