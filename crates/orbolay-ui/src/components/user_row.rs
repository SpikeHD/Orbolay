use freya::engine::prelude::SkColor;
use freya::prelude::*;

use orbolay_core::{
  app_state::AppState,
  user::{User, UserVoiceState},
};

use crate::{
  components::user_context_menu_item::UserContextMenuItem,
  util::{
    image::avatar_image,
    scale::{GapsScaleExt, UiScale},
    theme::Theme,
  },
};

static DEAFENED_SVG: &[u8] = include_bytes!("../../../../assets/deafened.svg");
static MUTED_SVG: &[u8] = include_bytes!("../../../../assets/muted.svg");
static STREAMING_SVG: &[u8] = include_bytes!("../../../../assets/streaming.svg");
static CAMERA_SVG: &[u8] = include_bytes!("../../../../assets/camera.svg");

#[derive(PartialEq)]
struct AvatarIcon {
  user: User,
  ui_scale: f32,
}

impl Component for AvatarIcon {
  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    let (url, border) = avatar_url_and_border(&self.user);
    rect()
      .width(Size::px(scale.px(50.0)))
      .height(Size::px(scale.px(50.0)))
      .corner_radius(CornerRadius::new_all(scale.px(25.0)))
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
  ui_scale: f32,
}

impl Component for UserLabel {
  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    let user = &self.user;
    let is_muted = user.voice_state == UserVoiceState::Muted;
    let is_deafened = user.voice_state == UserVoiceState::Deafened;

    rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Center)
      .cross_align(Alignment::Center)
      .height(Size::percent(70.0_f32))
      .background(self.theme.gray)
      .corner_radius(CornerRadius::new_all(self.theme.border_radius))
      .margin(Gaps::new(0., 6., 0., 6.).scaled(scale.factor()))
      .child(
        rect()
          .padding(Gaps::new(0., 6., 0., 6.).scaled(scale.factor()))
          .child(
            label()
              .font_size(scale.px(14.))
              .color(self.theme.text_color)
              .text(user.name.clone()),
          ),
      )
      .maybe(is_muted, |el| {
        el.child(
          SvgViewer::new(MUTED_SVG)
            .width(Size::px(scale.px(16.0)))
            .height(Size::px(scale.px(16.0)))
            .margin(Gaps::new(0., 6., 0., 0.).scaled(scale.factor())),
        )
      })
      .maybe(is_deafened, |el| {
        el.child(
          SvgViewer::new(DEAFENED_SVG)
            .width(Size::px(scale.px(16.0)))
            .height(Size::px(scale.px(16.0)))
            .margin(Gaps::new(0., 6., 0., 0.).scaled(scale.factor())),
        )
      })
      .maybe(user.streaming, |el| {
        el.child(
          SvgViewer::new(STREAMING_SVG)
            .width(Size::px(scale.px(16.0)))
            .height(Size::px(scale.px(16.0)))
            .margin(Gaps::new(0., 6., 0., 0.).scaled(scale.factor())),
        )
      })
      .maybe(user.camera, |el| {
        el.child(
          SvgViewer::new(CAMERA_SVG)
            .width(Size::px(scale.px(16.0)))
            .height(Size::px(scale.px(16.0)))
            .margin(Gaps::new(0., 6., 0., 0.).scaled(scale.factor())),
        )
      })
  }
}

#[derive(PartialEq)]
pub struct UserRow {
  pub app_state: State<AppState>,
  pub user: User,
  pub is_right_aligned: bool,
  pub is_open: bool,
  pub is_voice_semitransparent: bool,
  pub can_context_menu: bool,
  pub theme: Theme,
  pub ui_scale: f32,
}

impl Component for UserRow {
  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    let is_right_aligned = self.is_right_aligned;
    let is_speaking = self.user.voice_state == UserVoiceState::Speaking;
    let opacity = if !is_speaking && (self.is_voice_semitransparent && !self.is_open) {
      0.5_f32
    } else {
      1.0_f32
    };

    let label = UserLabel {
      user: self.user.clone(),
      theme: self.theme,
      ui_scale: scale.factor(),
    };
    let icon = AvatarIcon {
      user: self.user.clone(),
      ui_scale: scale.factor(),
    };

    let row = rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Start)
      .cross_align(Alignment::Center)
      .height(Size::px(scale.px(50.0)))
      .margin(Gaps::new_all(6.).scaled(scale.factor()))
      .opacity(opacity)
      .maybe(self.can_context_menu, |el| {
        el.on_secondary_down({
          let user = self.user.clone();
          let theme = self.theme;
          let app_state = self.app_state;
          move |e: Event<PressEventData>| {
            ContextMenu::open_from_event(
              &e,
              Menu::new()
                .theme(MenuContainerThemePartial {
                  background: Some(Preference::Specific(theme.darkish_gray)),
                  padding: Some(Preference::Specific(
                    Gaps::new_all(6.).scaled(scale.factor()),
                  )),
                  shadow: Some(Preference::Specific(theme.transparent_gray)),
                  border_fill: Some(Preference::Specific(theme.muted_gray)),
                  corner_radius: Some(Preference::Specific(CornerRadius::new_all(
                    theme.border_radius,
                  ))),
                })
                .child(UserContextMenuItem {
                  user: user.clone(),
                  theme,
                  app_state,
                  ui_scale: scale.factor(),
                }),
            );
          }
        })
      });

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
