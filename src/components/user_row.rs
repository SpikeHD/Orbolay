use freya::engine::prelude::SkColor;
use freya::prelude::*;

use crate::{
  app_state::AppState,
  config::{AxisAlignment, CornerAlignment},
  user::{User, UserVoiceState},
  util::{
    colors,
    image::{circular_with_border, fetch_icon, image_from_bytes},
  },
};

static DEAFENED_SVG: &[u8] = include_bytes!("../../assets/deafened.svg");
static MUTED_SVG: &[u8] = include_bytes!("../../assets/muted.svg");
static STREAMING_SVG: &[u8] = include_bytes!("../../assets/streaming.svg");

#[derive(PartialEq)]
struct AvatarIcon {
  user: User,
}

impl Component for AvatarIcon {
  fn render(&self) -> impl IntoElement {
    rect()
      .width(Size::px(50.))
      .height(Size::px(50.))
      .corner_radius(CornerRadius::new_all(25.))
      .child(
        image_from_bytes(avatar(&self.user))
          .width(Size::fill())
          .height(Size::fill()),
      )
  }
}

#[derive(PartialEq)]
struct UserLabel {
  user: User,
}

impl Component for UserLabel {
  fn render(&self) -> impl IntoElement {
    let user = &self.user;
    let is_muted = user.voice_state == UserVoiceState::Muted;
    let is_deafened = user.voice_state == UserVoiceState::Deafened;

    rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Center)
      .cross_align(Alignment::Center)
      .height(Size::percent(70.))
      .background(colors::GRAY)
      .corner_radius(CornerRadius::new_all(5.))
      .margin(Gaps::new(0., 6., 0., 6.))
      .child(
        rect()
          .padding(Gaps::new_all(4.))
          .child(label().font_size(14.).color(Color::WHITE).text(user.name.clone())),
      )
      .maybe(is_muted, |el| {
        el.child(
          svg(MUTED_SVG)
            .width(Size::px(16.))
            .height(Size::px(16.))
            .margin(Gaps::new(0., 6., 0., 0.)),
        )
      })
      .maybe(is_deafened, |el| {
        el.child(
          svg(DEAFENED_SVG)
            .width(Size::px(16.))
            .height(Size::px(16.))
            .margin(Gaps::new(0., 6., 0., 0.)),
        )
      })
      .maybe(user.streaming, |el| {
        el.child(
          svg(STREAMING_SVG)
            .width(Size::px(16.))
            .height(Size::px(16.))
            .margin(Gaps::new(0., 6., 0., 0.)),
        )
      })
  }
}

#[derive(PartialEq)]
pub struct UserRow {
  pub app_state: State<AppState>,
  pub user: User,
}

impl Component for UserRow {
  fn render(&self) -> impl IntoElement {
    let state = self.app_state.read();
    let alignment = CornerAlignment::from_str(&state.config.user_alignment);
    let is_right_aligned = alignment.x == AxisAlignment::End;
    let is_open = state.is_open;
    let is_voice_semitransparent = state.config.voice_semitransparent;
    let is_speaking = self.user.voice_state == UserVoiceState::Speaking;

    let opacity = if !is_speaking && (is_voice_semitransparent && !is_open) {
      0.5
    } else {
      1.0
    };

    rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Start)
      .cross_align(Alignment::Center)
      .height(Size::px(50.))
      .margin(Gaps::new_all(6.))
      .opacity(opacity)
      .maybe(is_right_aligned, |el| {
        el.child(UserLabel { user: self.user.clone() })
          .child(AvatarIcon { user: self.user.clone() })
      })
      .maybe(!is_right_aligned, |el| {
        el.child(AvatarIcon { user: self.user.clone() })
          .child(UserLabel { user: self.user.clone() })
      })
  }
}

fn avatar(user: &User) -> Vec<u8> {
  let border_color = match user.voice_state {
    UserVoiceState::Speaking => Some(SkColor::from_rgb(67, 147, 120)),
    UserVoiceState::Deafened | UserVoiceState::Muted => Some(SkColor::from_rgb(218, 62, 68)),
    _ => None,
  };

  if user.avatar.is_empty() {
    return circular_with_border(fetch_icon("", true).unwrap_or_default(), border_color)
      .unwrap_or_default();
  }

  let url = format!(
    "https://cdn.discordapp.com/avatars/{}/{}.png?size=80",
    user.id, user.avatar
  );

  circular_with_border(fetch_icon(&url, true).unwrap_or_default(), border_color).unwrap_or_default()
}
