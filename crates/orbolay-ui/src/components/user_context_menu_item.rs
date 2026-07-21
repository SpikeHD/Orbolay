use freya::prelude::*;
use serde_json::json;

use orbolay_core::{app_state::AppState, user::User, util::bridge::BridgeMessage};

use crate::util::{
  scale::{GapsScaleExt, UiScale},
  theme::Theme,
};

#[derive(PartialEq)]
pub struct UserContextMenuItem {
  pub user: User,
  pub theme: Theme,
  pub app_state: State<AppState>,
  pub ui_scale: f32,
}

impl Component for UserContextMenuItem {
  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    let mut app_state = self.app_state;
    let user_id = self.user.id.clone();
    let mut slider_value = use_state(|| f64::from(self.user.volume.clamp(0., 200.) / 2.));

    let menu_item_theme = MenuItemThemePartial {
      background: Some(Preference::Specific(self.theme.darkish_gray)),
      hover_background: Some(Preference::Specific(self.theme.darkish_gray)),
      select_background: Some(Preference::Specific(self.theme.darkish_gray)),
      border_fill: Some(Preference::Specific(Color::TRANSPARENT)),
      select_border_fill: Some(Preference::Specific(Color::TRANSPARENT)),
      corner_radius: Some(Preference::Specific(CornerRadius::new_all(
        self.theme.border_radius,
      ))),
      color: Some(Preference::Specific(self.theme.text_color)),
    };

    let slider_theme = SliderThemePartial {
      background: Some(Preference::Specific(self.theme.muted_gray)),
      thumb_background: Some(Preference::Specific(self.theme.text_color)),
      thumb_inner_background: Some(Preference::Specific(self.theme.text_color)),
      border_fill: Some(Preference::Specific(self.theme.gray)),
    };

    MenuItem::new()
      .theme(menu_item_theme)
      .padding(Gaps::new_all(4.).scaled(scale.factor()))
      .child(
        rect()
          .direction(Direction::Vertical)
          .cross_align(Alignment::Start)
          .width(Size::px(scale.px(160.0)))
          .child(
            rect()
              .direction(Direction::Horizontal)
              .main_align(Alignment::SpaceBetween)
              .cross_align(Alignment::Center)
              .width(Size::fill())
              .margin(Gaps::new(0., 0., 8., 0.).scaled(scale.factor()))
              .child(
                label()
                  .font_size(scale.px(12.))
                  .color(self.theme.text_color)
                  .text("User Volume"),
              )
              .child(
                label()
                  .font_size(scale.px(12.))
                  .color(self.theme.text_color)
                  .text(format!("{}%", slider_value.read().round() as u64 * 2)),
              ),
          )
          .child(
            Slider::new(move |value: f64| {
              let volume = (value * 2.0).clamp(0.0, 200.0);
              slider_value.set(value);
              app_state.write().send(BridgeMessage {
                cmd: "SET_USER_VOLUME".to_string(),
                data: json!({
                  "user_id": user_id,
                  "volume": volume,
                }),
              });
            })
            .theme(slider_theme)
            .value(*slider_value.read())
            .size(Size::fill()),
          ),
      )
  }
}
