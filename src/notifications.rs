use orbolay_core::app_state::AppHandle;

pub fn create_notification_thread(app: AppHandle) {
  std::thread::spawn(move || {
    loop {
      std::thread::sleep(std::time::Duration::from_secs(1));

      let current_timestamp = chrono::Utc::now().timestamp();

      app.update_if_changed(|state| {
        let before = state.messages.len();

        state.messages.retain(|message| {
          if let Some(ts) = message.timestamp {
            return current_timestamp - ts < message.timeout_secs;
          }
          true
        });

        before != state.messages.len()
      });
    }
  });
}
