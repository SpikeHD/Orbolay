use freya::engine::prelude::SkColor;
use freya::prelude::*;

use crate::{
  user::{User, UserVoiceState},
  util::{image::avatar_image, theme::Theme},
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
    let (url, border) = avatar_url_and_border(&self.user);
    rect()
      .width(Size::px(50.))
      .height(Size::px(50.))
      .corner_radius(CornerRadius::new_all(25.))
      .child(
        avatar_image(&url, border)
          .width(Size::fill())
          .height(Size::fill()),
      )
  }
}

#[derive(PartialEq)]
struct UserLabel {
  user: User,
  theme: Theme,
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
      .background(self.theme.gray)
      .corner_radius(CornerRadius::new_all(self.theme.border_radius))
      .margin(Gaps::new(0., 6., 0., 6.))
      .child(
        rect().padding(Gaps::new_all(4.)).child(
          label()
            .font_size(14.)
            .color(self.theme.text_color)
            .text(user.name.clone()),
        ),
      )
      .maybe(is_muted, |el| {
        el.child(
          SvgViewer::new(MUTED_SVG)
            .width(Size::px(16.))
            .height(Size::px(16.))
            .margin(Gaps::new(0., 6., 0., 0.)),
        )
      })
      .maybe(is_deafened, |el| {
        el.child(
          SvgViewer::new(DEAFENED_SVG)
            .width(Size::px(16.))
            .height(Size::px(16.))
            .margin(Gaps::new(0., 6., 0., 0.)),
        )
      })
      .maybe(user.streaming, |el| {
        el.child(
          SvgViewer::new(STREAMING_SVG)
            .width(Size::px(16.))
            .height(Size::px(16.))
            .margin(Gaps::new(0., 6., 0., 0.)),
        )
      })
  }
}

#[derive(PartialEq)]
pub struct UserRow {
  pub user: User,
  pub is_right_aligned: bool,
  pub is_open: bool,
  pub is_voice_semitransparent: bool,
  pub theme: Theme,
}

impl Component for UserRow {
  fn render(&self) -> impl IntoElement {
    let is_right_aligned = self.is_right_aligned;
    let is_speaking = self.user.voice_state == UserVoiceState::Speaking;

    let opacity = if !is_speaking && (self.is_voice_semitransparent && !self.is_open) {
      0.5
    } else {
      1.0
    };

    let label = UserLabel {
      user: self.user.clone(),
      theme: self.theme,
    };
    let icon = AvatarIcon {
      user: self.user.clone(),
    };

    let row = rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Start)
      .cross_align(Alignment::Center)
      .height(Size::px(50.))
      .margin(Gaps::new_all(6.))
      .opacity(opacity);

    if is_right_aligned {
      row.child(label).child(icon)
    } else {
      row.child(icon).child(label)
    }
  }
}

fn avatar_url_and_border(user: &User) -> (String, Option<SkColor>) {
  let border_color = match user.voice_state {
    UserVoiceState::Speaking => Some(SkColor::from_rgb(67, 147, 120)),
    UserVoiceState::Deafened | UserVoiceState::Muted => Some(SkColor::from_rgb(218, 62, 68)),
    _ => None,
  };

  let url = if user.avatar.is_empty() {
    String::new()
  } else {
    format!(
      "https://cdn.discordapp.com/avatars/{}/{}.png?size=80",
      user.id, user.avatar
    )
  };

  (url, border_color)
}
