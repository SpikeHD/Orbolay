use freya::prelude::*;

use orbolay_core::{
  app_state::AppState,
  config::{DisplayVoiceMembers, TransportMode},
  user::{User, UserVoiceState},
  util::text::censor,
};

use crate::{
  components::UserRow,
  config::{AxisAlignment, CornerAlignment},
  util::theme::Theme,
};

#[derive(PartialEq)]
pub struct VoiceSection {
  pub app_state: State<AppState>,
  pub voice_users: Vec<User>,
  pub is_open: bool,
  pub is_censor: bool,
  pub user_alignment: String,
  pub user_offset_x: i32,
  pub user_offset_y: i32,
  pub display_voice_members: DisplayVoiceMembers,
  pub theme: Theme,
}

impl Component for VoiceSection {
  fn render(&self) -> impl IntoElement {
    let alignment = CornerAlignment::from_str(&self.user_alignment);
    let gaps = alignment.to_gaps(self.user_offset_x, self.user_offset_y);
    let is_right_aligned = alignment.x == AxisAlignment::End;

    let mut sorted_users = self.voice_users.clone();
    sorted_users.sort_by(|a, b| a.id.cmp(&b.id));

    // Filter users based on display_voice_members
    let filtered_users: Vec<_> = sorted_users
      .into_iter()
      .filter(|user| match self.display_voice_members {
        DisplayVoiceMembers::Always => true,
        DisplayVoiceMembers::AlwaysSemiTransparent => true,
        DisplayVoiceMembers::WhenSpeaking => {
          user.voice_state == UserVoiceState::Speaking || self.is_open
        }
      })
      .collect();

    rect().child(ContextMenuViewer::new()).child(
      filtered_users.iter().fold(
        rect()
          .direction(Direction::Vertical)
          .cross_align(alignment.x.to_freya())
          .main_align(alignment.y.to_freya())
          .position(Position::new_absolute().top(0.).left(0.))
          .background(Color::TRANSPARENT)
          .height(Size::fill())
          .width(Size::fill())
          .padding(gaps),
        |el, user| {
          let mut u = user.clone();

          if self.is_censor {
            u.name = censor(&u.name);
          }

          el.child(UserRow {
            app_state: self.app_state,
            // TODO websocket cannot change user volume yet
            can_context_menu: self.app_state.read().user_id != u.id.clone()
              && self.app_state.read().config.transport_mode == TransportMode::Ipc,
            user: u,
            is_open: self.is_open,
            is_right_aligned,
            is_voice_semitransparent: matches!(
              self.display_voice_members,
              DisplayVoiceMembers::AlwaysSemiTransparent
            ),
            theme: self.theme,
          })
        },
      ),
    )
  }
}
