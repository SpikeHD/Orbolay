use orbolay_core::{app_state::AppHandle, payloads::Notification};

pub fn create_notification_thread(app: AppHandle) {
  std::thread::spawn(move || {
    loop {
      std::thread::sleep(std::time::Duration::from_secs(1));

      let current_timestamp = chrono::Utc::now().timestamp();
      let alive = |message: &Notification| {
        message
          .timestamp
          .is_none_or(|ts| current_timestamp - ts < message.timeout_secs)
      };

      app.update_if_changed(|state| {
        let before = state.messages.len() + state.chat_messages.len();

        state.messages.retain(|m| alive(m));
        let chat_on = state.config.voice_chat_enabled();
        let channel = state.current_channel.clone();
        let current =
          |m: &Notification| chat_on && m.channel_id.as_deref() == Some(channel.as_str());
        state.chat_messages.retain(|m| current(m) && alive(m));
        state.pending_chat.retain(|m| current(m));

        let shown = state.show_pending();
        shown || before != state.messages.len() + state.chat_messages.len()
      });
    }
  });
}
