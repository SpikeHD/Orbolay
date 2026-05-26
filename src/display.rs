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

pub fn update_monitor() {
  let display = specific_monitor_or_primary();
  let monitor_position = (display.x, display.y);
  let monitor_size = (display.width, display.height);

  Platform::get().with_window(None, move |w| {
    w.set_outer_position(PhysicalPosition::new(
      monitor_position.0,
      monitor_position.1,
    ));
    w.set_min_inner_size(Some(PhysicalSize::new(
      (monitor_size.0 + 1) as f64,
      (monitor_size.1 - 1) as f64,
    )));
  });
}
