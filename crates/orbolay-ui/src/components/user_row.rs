use freya::{
  html::{HtmlSource, HtmlViewer, use_html},
  prelude::*,
};

use orbolay_core::{
  app_state::AppState,
  user::{User, UserVoiceState},
};

use crate::{
  components::user_context_menu_item::UserContextMenuItem,
  util::{
    scale::{GapsScaleExt, UiScale},
    theme::Theme,
    user_template::load_user_template,
  },
};

#[derive(PartialEq, Clone)]
pub struct UserRow {
  pub app_state: State<AppState>,
  pub user: User,
  pub is_self: bool,
  pub is_open: bool,
  pub is_voice_semitransparent: bool,
  pub can_context_menu: bool,
  pub theme: Theme,
  pub box_size: (u32, u32),
  pub user_template: Option<String>,
  pub is_right_aligned: bool,
  pub ui_scale: f32,
}

impl Component for UserRow {
  // Stable diff key based on user ID
  fn render_key(&self) -> DiffKey {
    DiffKey::from(&self.user.id)
  }

  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    let width = scale.px(self.box_size.0 as f32);
    let height = scale.px(self.box_size.1 as f32);
    let opacity = if self.user.voice_state != UserVoiceState::Speaking
      && self.is_voice_semitransparent
      && !self.is_open
    {
      0.5_f32
    } else {
      1.0_f32
    };

    let handle = use_html(|| HtmlSource::html(String::new()));
    let deps = (
      self.user.clone(),
      self.is_self,
      self.theme,
      self.user_template.clone(),
      self.is_right_aligned,
    );

    use_side_effect_with_deps(&deps, move |deps| {
      let (user, is_self, theme, user_template, is_right_aligned) = deps.clone();
      let template = load_user_template(user_template.as_deref());
      let mut handle = handle;
      handle.load_html(template.render(&user, is_self, &theme, is_right_aligned));
    });

    rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Center)
      .cross_align(Alignment::Center)
      .width(Size::px(width))
      .height(Size::px(height))
      .margin(Gaps::new_all(6.).scaled(scale.factor()))
      .opacity(opacity)
      .child(
        rect()
          .width(Size::px(self.box_size.0 as f32))
          .height(Size::px(self.box_size.1 as f32))
          .scale(self.ui_scale)
          .child(HtmlViewer::new(handle)),
      )
      .maybe(self.can_context_menu, |el| {
        el.on_secondary_down({
          let user = self.user.clone();
          let theme = self.theme;
          let app_state = self.app_state;
          move |_| {
            ContextMenu::open_from_down(
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
      })
  }
}
