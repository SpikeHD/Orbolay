use serde_json::Value;

use crate::{app_state::SharedAppState, payloads::Notification, warn};

pub fn maybe_notify_update(shared: SharedAppState) {
  std::thread::spawn(move || {
    let Ok(mut resp) =
      ureq::get("https://api.github.com/repos/SpikeHD/orbolay/releases/latest").call()
    else {
      warn!("Failed to check for updates: request failed");
      return;
    };
    let Ok(body) = resp.body_mut().read_to_string() else {
      warn!("Failed to check for updates: could not read response body");
      return;
    };
    let Ok(json) = serde_json::from_str::<Value>(&body) else {
      warn!("Failed to check for updates: could not parse response as JSON");
      return;
    };

    if let Some(latest_version) = json.get("tag_name").and_then(|v| v.as_str()) {
      let current_version = format!("v{}", env!("CARGO_PKG_VERSION"));
      if latest_version != current_version {
        shared.write().unwrap().notify(Notification {
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
