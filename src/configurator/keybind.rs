use freya::prelude::*;
use rdev::Key;

#[derive(PartialEq)]
pub struct KeybindControl {
  initial: Option<String>,
  on_change: EventHandler<Vec<Key>>,
  on_focus: EventHandler<()>,
  on_blur: EventHandler<()>,
}

impl KeybindControl {
  pub fn new(initial: Option<String>, on_change: EventHandler<Vec<Key>>, on_focus: EventHandler<()>, on_blur: EventHandler<()>) -> Self {
    Self { initial, on_change, on_focus, on_blur }
  }
}

impl Component for KeybindControl {
  fn render(&self) -> impl IntoElement {
    let theme_color = get_theme!(
      None,
      InputColorsThemePreference,
      "input"
    );
    let theme_layout = get_theme!(
      None,
      InputLayoutThemePreference,
      "input_layout"
    );
    let id = use_a11y();
    let focus_status = use_focus(id);
    let value = use_state(|| self.initial.clone().unwrap_or_default());

    let on_focus = self.on_focus.clone();
    let on_blur = self.on_blur.clone();

    let on_press = move |_| {
      id.request_focus();
    };
    let on_global_press = move |_| {
      if !focus_status.read().is_focused() {
        return;
      }

      id.request_unfocus();
    };
    let on_key_down = move |event: Event<KeyboardEventData>| {
      if !focus_status.read().is_focused() {
        return;
      }

      println!("Key down: {:?}", event.data());
    };
    let on_key_up = move |event: Event<KeyboardEventData>| {
      if !focus_status.read().is_focused() {
        return;
      }

      println!("Key up: {:?}", event.data());
    };

    let border = if focus_status.read().is_focused() {
      Border::new().fill(theme_color.focus_border_fill).width(2.0)
    } else {
      Border::new().fill(theme_color.border_fill).width(1.0)
    };

    use_side_effect(move || {
      if focus_status.read().is_focused() {
        on_focus.call(());
      } else {
        on_blur.call(());
      }
    });

    rect()
      .a11y_id(id)
      .a11y_focusable(true)
      .on_press(on_press)
      .on_global_pointer_press(on_global_press)
      .on_global_key_down(on_key_down)
      .on_global_key_up(on_key_up)
      .background(theme_color.background)
      .border(border)
      .corner_radius(theme_layout.corner_radius)
      .color(theme_color.color)
      .width(Size::px(200.0))
      .height(Size::px(32.0))
  }
}
