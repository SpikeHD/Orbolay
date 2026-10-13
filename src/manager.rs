use freya::prelude::State;

use orbolay_core::app_state::AppState;

pub struct OverlayManager;

impl OverlayManager {
  pub fn set_state(mut app_state: State<AppState>, is_open: bool) {
    app_state.write().is_open = is_open;
  }

  pub fn close(app_state: State<AppState>) {
    Self::set_state(app_state, false);
  }

  pub fn toggle(mut app_state: State<AppState>) {
    let mut state = app_state.write();
    state.is_open = !state.is_open;
  }

  #[allow(dead_code)]
  pub fn open(app_state: State<AppState>) {
    Self::set_state(app_state, true);
  }
}
