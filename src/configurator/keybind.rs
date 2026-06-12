use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use freya::animation::*;
use freya::prelude::*;
use rdev::Key;

use crate::keys::{bind::key_to_string, convert::code_to_rdev};

pub fn keys_display(keys: &[Key]) -> String {
  keys
    .iter()
    .map(key_to_string)
    .collect::<Vec<_>>()
    .join(" + ")
}

#[derive(PartialEq)]
pub struct KeybindControl {
  initial: Option<Vec<Key>>,
  on_change: EventHandler<Vec<Key>>,
}

impl KeybindControl {
  pub fn new(initial: Option<Vec<Key>>, on_change: EventHandler<Vec<Key>>) -> Self {
    Self { initial, on_change }
  }
}

impl Component for KeybindControl {
  fn render(&self) -> impl IntoElement {
    let theme_color = get_theme!(None, InputColorsThemePreference, "input");
    let theme_layout = get_theme!(None, InputLayoutThemePreference, "input_layout");
    let id = use_a11y();
    let focus_status = use_focus(id);

    let focus_border_fill = theme_color.focus_border_fill;
    let border_anim = use_animation(move |conf| {
      conf.on_finish(OnFinish::reverse());
      AnimColor::new(focus_border_fill, Color::from_rgb(200, 50, 50)).time(500)
    });

    let recording_flag = use_try_consume::<Arc<AtomicBool>>();

    use_side_effect(move || {
      let is_focused = focus_status.read().is_focused();
      if let Some(flag) = &recording_flag {
        flag.store(is_focused, Ordering::Relaxed);
      }
      let mut anim = border_anim;
      if is_focused {
        anim.start();
      } else {
        anim.reset();
      }
    });

    // Keys currently being physically held down
    let mut pressing = use_state(Vec::<Key>::new);
    // All keys pressed in the current recording session
    let mut candidate = use_state(Vec::<Key>::new);
    let mut recorded = use_state(|| self.initial.clone().unwrap_or_default());

    let on_change = self.on_change.clone();

    let mut hovered = use_state(|| false);

    use_drop(move || {
      if *hovered.read() {
        Cursor::set(CursorIcon::default());
      }
    });

    let on_press = move |_| id.request_focus();

    let on_global_press = move |_| {
      if focus_status.read().is_focused() {
        pressing.write().clear();
        candidate.write().clear();
        id.request_unfocus();
      }
    };

    let on_key_down = move |event: Event<KeyboardEventData>| {
      if !focus_status.read().is_focused() {
        return;
      }

      let Some(key) = code_to_rdev(event.data().code) else {
        return;
      };

      event.prevent_default();
      event.stop_propagation();

      let mut p = pressing.write();
      if !p.contains(&key) {
        p.push(key);
      }

      let mut c = candidate.write();
      if !c.contains(&key) {
        c.push(key);
      }
    };

    let on_key_up = move |event: Event<KeyboardEventData>| {
      if !focus_status.read().is_focused() {
        return;
      }

      let Some(key) = code_to_rdev(event.data().code) else {
        return;
      };

      pressing.write().retain(|k| k != &key);

      if pressing.read().is_empty() {
        let keys = candidate.read().clone();
        if !keys.is_empty() {
          *recorded.write() = keys.clone();
          *candidate.write() = vec![];
          on_change.call(keys);
        }
      }
    };

    let is_focused = focus_status.read().is_focused();
    let candidate_keys = candidate.read().clone();
    let recorded_keys = recorded.read().clone();

    let (display_text, is_placeholder) = if is_focused && !candidate_keys.is_empty() {
      (keys_display(&candidate_keys), false)
    } else if !recorded_keys.is_empty() {
      (keys_display(&recorded_keys), false)
    } else if is_focused {
      ("Press keys...".into(), true)
    } else {
      (String::new(), false)
    };

    let border = if is_focused {
      Border::new().fill(border_anim.read().value()).width(2.0)
    } else {
      Border::new().fill(theme_color.border_fill).width(1.0)
    };

    let text_color = if is_placeholder {
      theme_color.placeholder_color
    } else {
      theme_color.color
    };

    rect()
      .a11y_id(id)
      .a11y_focusable(true)
      .on_press(on_press)
      .on_pointer_enter(move |_| {
        *hovered.write() = true;
        Cursor::set(CursorIcon::Pointer);
      })
      .on_pointer_leave(move |_| {
        *hovered.write() = false;
        Cursor::set(CursorIcon::default());
      })
      .on_global_pointer_press(on_global_press)
      .on_global_key_down(on_key_down)
      .on_global_key_up(on_key_up)
      .background(theme_color.background)
      .border(border)
      .corner_radius(theme_layout.corner_radius)
      .padding(theme_layout.inner_margin)
      .width(Size::px(300.0))
      .height(Size::px(32.0))
      .main_align(Alignment::Center)
      .cross_align(Alignment::Start)
      .child(label().color(text_color).font_size(16.0).text(display_text))
  }
}
