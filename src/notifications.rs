use dioxus::signals::{Signal, SyncStorage};
use freya::prelude::*;

use crate::app_state::AppState;

pub fn create_notification_thread(mut app_state: Signal<AppState, SyncStorage>) {
  std::thread::spawn(move || {
    loop {
      std::thread::sleep(std::time::Duration::from_secs(1));

      let current_timestamp = chrono::Utc::now().timestamp();
      app_state.write().messages.retain(|message| {
        let msg = message.clone();
        if let Some(message_timestamp) = msg.timestamp {
          let timestamp = message_timestamp.parse::<i64>().unwrap_or(0);
          return current_timestamp - timestamp < 5;
        }

        true
      });
    }
  });
}
