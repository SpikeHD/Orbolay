#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use freya::prelude::*;
use winit::window::WindowLevel;

use crate::components::user_row::user_row;

mod components;

fn main() {
  launch_cfg(
    app,
    LaunchConfig::<f32>::new()
      .with_decorations(false)
      .with_background("transparent")
      .with_transparency(true)
      .with_window_attributes(|w| {
        w.with_window_level(WindowLevel::AlwaysOnTop)
          .with_resizable(false)
      }),
  );
}

fn app() -> Element {
  let platform = use_platform();

  platform.with_window(|w| {
    // Disable hittest
    w.set_cursor_hittest(false).unwrap_or_default();
  });

  rsx!(
    rect {
      content: "flex",
      direction: "vertical",
  
      background: "transparent",
      height: "600",
      width: "200",

      user_row {
        speaking: true,
        name: "John Doe".to_string(),
        avatar: vec![0],
      }

      user_row {
        speaking: false,
        name: "Jane Doe".to_string(),
        avatar: vec![1],
      }

      user_row {
        speaking: true,
        name: "Jebediah Doe".to_string(),
        avatar: vec![2],
      }
    }
  )
}
