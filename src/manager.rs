use crate::app_state::AppState;
use freya::prelude::*;

pub struct OverlayManager;

impl OverlayManager {
  pub fn toggle(app_state: &mut Signal<AppState, SyncStorage>, platform: &PlatformSender) {
    let new_state = !app_state.read().is_open;
    Self::set_state(app_state, platform, new_state);
  }

  pub fn set_state(
    app_state: &mut Signal<AppState, SyncStorage>,
    platform: &PlatformSender,
    is_open: bool,
  ) {
    app_state.write().is_open = is_open;

    platform.with_window(move |w| {
      let _ = w.set_cursor_hittest(is_open);
    });
  }

  pub fn close(app_state: &mut Signal<AppState, SyncStorage>, platform: &PlatformSender) {
    Self::set_state(app_state, platform, false);
  }

  #[allow(dead_code)]
  pub fn open(app_state: &mut Signal<AppState, SyncStorage>, platform: &PlatformSender) {
    Self::set_state(app_state, platform, true);
  }
}
