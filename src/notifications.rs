use crate::app_state::SharedAppState;

pub fn create_notification_thread(shared: SharedAppState, redraw_tx: flume::Sender<()>) {
  std::thread::spawn(move || {
    loop {
      std::thread::sleep(std::time::Duration::from_secs(1));

      let current_timestamp = chrono::Utc::now().timestamp();
      {
        let mut state = shared.write().unwrap();
        let before = state.messages.len();

        state.messages.retain(|message| {
          let msg = message.clone();
          if let Some(message_timestamp) = msg.timestamp {
            let timestamp = message_timestamp.parse::<i64>().unwrap_or(0);
            return current_timestamp - timestamp < 5;
          }
          true
        });

        let after = state.messages.len();

        if before != after {
          let _ = redraw_tx.send(());
        }
      }
    }
  });
}
