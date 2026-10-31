use std::time::Duration;

use freya::prelude::*;

use crate::util::theme::{DARKISH_GRAY, LIGHT_GRAY, MUTED_GRAY, RED};

const CONFIRM_TIMEOUT: Duration = Duration::from_secs(5);

fn action_button(
  text: &'static str,
  background: Color,
  hover_background: Color,
  color: Color,
  on_press: impl Into<EventHandler<Event<PressEventData>>>,
) -> impl IntoElement {
  Button::new()
    .width(Size::fill())
    .height(Size::px(32.0_f32))
    .corner_radius(8.)
    .background(background)
    .hover_background(hover_background)
    .border_fill(Color::TRANSPARENT)
    .cursor(CursorIcon::Pointer)
    .on_press(on_press)
    .child(label().text(text).color(color).font_size(13.))
}

pub fn secondary_button(
  text: &'static str,
  on_press: impl Into<EventHandler<Event<PressEventData>>>,
) -> impl IntoElement {
  action_button(text, LIGHT_GRAY, MUTED_GRAY, Color::WHITE, on_press)
}

pub fn wide_button(
  text: &'static str,
  on_press: impl Into<EventHandler<Event<PressEventData>>>,
) -> Element {
  rect()
    .width(Size::fill())
    .padding(Gaps::new_symmetric(8., 12.))
    .child(secondary_button(text, on_press))
    .into_element()
}

#[derive(PartialEq)]
pub struct ResetButton {
  pub on_reset: EventHandler<()>,
}

impl Component for ResetButton {
  fn render(&self) -> impl IntoElement {
    let on_reset = self.on_reset.clone();
    let mut armed = use_state(|| false);
    let mut timeout = use_state(|| None::<TaskHandle>);

    let (text, background, hover_background, color) = if armed() {
      ("Are you sure?", RED, RED, Color::WHITE)
    } else {
      ("Reset to Defaults", DARKISH_GRAY, LIGHT_GRAY, RED)
    };

    action_button(text, background, hover_background, color, move |_| {
      if let Some(task) = timeout.write().take() {
        task.try_cancel();
      }

      if armed() {
        armed.set(false);
        on_reset.call(());
        return;
      }

      armed.set(true);
      timeout.set(Some(spawn(async move {
        timer(CONFIRM_TIMEOUT).await;
        armed.set(false);
      })));
    })
  }
}
