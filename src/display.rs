use display_info::DisplayInfo;
use freya::prelude::{Platform, WinitPlatformExt};
use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::{config::load_config, warn};

pub fn specific_monitor_or_primary() -> DisplayInfo {
  let config = load_config().unwrap_or_default();
  let displays = DisplayInfo::all().unwrap_or_default();

  if let Some(idx) = config.display_idx {
    if idx >= displays.len() {
      warn!(
        "Saved display index {} is out of bounds ({} displays detected)",
        idx,
        displays.len()
      );

      displays
        .iter()
        .find(|m| m.is_primary)
        .unwrap_or(displays.first().expect("No displays found"))
        .clone()
    } else {
      displays[idx].clone()
    }
  } else {
    displays
      .iter()
      .find(|m| m.is_primary)
      .unwrap_or(displays.first().expect("No displays found"))
      .clone()
  }
}

pub fn window_size_for_display(display: &DisplayInfo) -> PhysicalSize<f64> {
  let monitor_size = (display.width, display.height);

  // https://discourse.glfw.org/t/black-screen-when-setting-window-to-transparent-and-size-to-1920x1080/2585/5
  // We do -1 specifically on the height to fix the Windows hidden taskbar thing
  PhysicalSize::new(
    (monitor_size.0 + 1) as f64 * display.scale_factor as f64,
    (monitor_size.1 - 1) as f64 * display.scale_factor as f64,
  )
}

pub fn update_monitor() {
  let display = specific_monitor_or_primary();
  let monitor_position = (display.x, display.y);

  let new_size = window_size_for_display(&display);

  Platform::get().with_window(None, move |w| {
    w.set_outer_position(PhysicalPosition::new(
      monitor_position.0,
      monitor_position.1,
    ));

    let _ = w.request_inner_size(new_size);
  });
}
