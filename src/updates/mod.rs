use serde_json::Value;

use crate::{app_state::SharedAppState, payloads::MessageNotification};

pub fn maybe_notify_update(shared: SharedAppState) {
  std::thread::spawn(move || {
    let mut resp = ureq::get("https://api.github.com/repos/SpikeHD/orbolay/releases/latest")
      .call()
      .expect("Failed to get latest Orbolay release");
    let json = resp
      .body_mut()
      .read_to_string()
      .expect("Failed to read response body");
    let json = serde_json::from_str::<Value>(&json).expect("Failed to parse response body as JSON");

    if let Some(latest_version) = json.get("tag_name").and_then(|v| v.as_str()) {
      let current_version = env!("CARGO_PKG_VERSION");
      if latest_version != current_version {
        shared.write().unwrap().notify(MessageNotification {
          title: "Update Available!".into(),
          body:
            "An new update of Orbolay is available and can be downloaded via the GitHub releases"
              .into(),
          icon: "https://avatars.githubusercontent.com/u/25207995?v=4".to_string(),
          ..Default::default()
        });
      }
    }
  });
}
