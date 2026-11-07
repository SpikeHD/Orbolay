use freya::prelude::{Platform, WinitPlatformExt};
use winit::{dpi::PhysicalSize, monitor::MonitorHandle};

use orbolay_core::monitors::{MonitorInfo, pick_display, set_monitors};
use orbolay_logging::warn;

use crate::window::WINDOW_ID;

pub fn select_monitor<'a>(
  primary: Option<&MonitorHandle>,
  monitors: &'a [MonitorHandle],
  display_name: Option<&str>,
  display_idx: Option<usize>,
) -> Option<&'a MonitorHandle> {
  if let Some(idx) = display_idx
    && idx >= monitors.len()
  {
    warn!(
      "Saved display index {} is out of bounds ({} displays detected)",
      idx,
      monitors.len()
    );
  }

  let primary_idx = primary.and_then(|p| monitors.iter().position(|m| m == p));
  let idx = pick_display(
    monitors.len(),
    |idx| monitor_id(&monitors[idx]),
    primary_idx,
    display_name,
    display_idx,
  );
  monitors.get(idx.unwrap_or(0))
}

#[cfg(target_os = "macos")]
fn monitor_id(monitor: &MonitorHandle) -> Option<String> {
  use winit::platform::macos::MonitorHandleExtMacOS;

  Some(format!("Display {}", monitor.native_id()))
}

#[cfg(not(target_os = "macos"))]
fn monitor_id(monitor: &MonitorHandle) -> Option<String> {
  monitor.name()
}

pub fn window_size_for_display(monitor: &MonitorHandle) -> PhysicalSize<u32> {
  let size = monitor.size();

  // https://discourse.glfw.org/t/black-screen-when-setting-window-to-transparent-and-size-to-1920x1080/2585/5
  // We do -1 specifically on the height to fix the Windows hidden taskbar thing
  PhysicalSize::new(size.width + 1, size.height.saturating_sub(1))
}

/// Share the monitor list so it matches up everywhere
pub fn populate_monitor_cache(primary: Option<&MonitorHandle>, monitors: &[MonitorHandle]) {
  let infos: Vec<MonitorInfo> = monitors
    .iter()
    .enumerate()
    .map(|(idx, m)| {
      let size = m.size();
      let id = monitor_id(m).unwrap_or_else(|| format!("Display {idx}"));
      MonitorInfo {
        name: id.clone(),
        id,
        is_primary: primary.map(|p| p == m).unwrap_or(idx == 0),
        width: size.width,
        height: size.height,
      }
    })
    .collect();

  set_monitors(infos);
}

pub fn update_monitor(display_name: Option<String>, display_idx: Option<usize>) {
  let Some(window_id) = *WINDOW_ID.lock().unwrap() else {
    return;
  };
  Platform::get().with_window(window_id, move |w| {
    let monitors: Vec<_> = w.available_monitors().collect();
    let primary = w.primary_monitor();

    populate_monitor_cache(primary.as_ref(), &monitors);

    let Some(monitor) = select_monitor(
      primary.as_ref(),
      &monitors,
      display_name.as_deref(),
      display_idx,
    ) else {
      return;
    };

    let size = window_size_for_display(monitor);
    let positioned = w
      .outer_position()
      .ok()
      .is_none_or(|pos| pos == monitor.position());
    if positioned && w.inner_size() == size {
      return;
    }

    w.set_outer_position(monitor.position());
    let _ = w.request_inner_size(size);
  });
}
