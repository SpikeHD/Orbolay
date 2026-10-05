use freya::{
  html::{HtmlSource, HtmlViewer, use_html},
  prelude::*,
};

use orbolay_core::{app_state::AppState, payloads::Notification, util::bridge::BridgeMessage};

use crate::util::{
  notification_template::{self, load_notification_template},
  scale::{GapsScaleExt, UiScale},
  theme::Theme,
};

#[derive(PartialEq)]
pub struct MessageRow {
  pub app_state: State<AppState>,
  pub message: Notification,
  pub theme: Theme,
  pub box_size: (u32, u32),
  pub notification_template: Option<String>,
  pub ui_scale: f32,
}

impl Component for MessageRow {
  fn render_key(&self) -> DiffKey {
    if let Some(message_id) = &self.message.message_id {
      DiffKey::from(message_id)
    } else {
      DiffKey::from(&self.message.title)
    }
  }

  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    let width = scale.px(self.box_size.0 as f32);
    let height = scale.px(self.box_size.1 as f32);
    let message = self.message.clone();
    let ui_scale = self.ui_scale;
    let mut app_state = self.app_state;

    let handle = use_html(|| HtmlSource::html(String::new()));
    let deps = (
      self.message.clone(),
      self.theme,
      self.notification_template.clone(),
    );

    use_side_effect_with_deps(&deps, move |deps| {
      let (message, theme, notification_template) = deps.clone();
      let template = load_notification_template(notification_template.as_deref());
      let mut handle = handle;
      handle.load_html(template.render(&message, &theme));
    });

    rect()
      .direction(Direction::Horizontal)
      .main_align(Alignment::Center)
      .cross_align(Alignment::Center)
      .width(Size::px(width))
      .height(Size::px(height))
      .margin(Gaps::new_all(6.).scaled(scale.factor()))
      .cursor(CursorIcon::Pointer)
      .child(
        rect()
          .width(Size::px(self.box_size.0 as f32))
          .height(Size::px(self.box_size.1 as f32))
          .scale(self.ui_scale)
          .child(HtmlViewer::new(handle)),
      )
      .on_press(move |event: Event<PressEventData>| {
        let (mouse_x, mouse_y) = match &*event {
          PressEventData::Mouse(m) => (m.element_location.x, m.element_location.y),
          PressEventData::Touch(t) => (t.element_location.x, t.element_location.y),
          PressEventData::Keyboard(_) => return,
        };
        let local_x = (mouse_x as f32) / ui_scale;
        let local_y = (mouse_y as f32) / ui_scale;
        let hits = handle.elements_at(local_x, local_y);

        for hit in hits {
          if let Some(index) = hit
            .attr(notification_template::ACTION_ATTR)
            .and_then(|value| value.parse::<usize>().ok())
            && let Some(actions) = &message.actions
            && let Some(action) = actions.get(index)
          {
            (action.action)();
            return;
          }
        }

        app_state.write().send(BridgeMessage {
          cmd: "NAVIGATE".to_string(),
          data: serde_json::json!({
            "guild_id": message.guild_id,
            "channel_id": message.channel_id,
            "message_id": message.message_id,
          }),
        })
      })
  }
}
