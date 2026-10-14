use serde_json::Value;

use orbolay_core::{app_state::AppHandle, payloads::Notification};
use orbolay_logging::warn;

pub fn maybe_notify_update(app: AppHandle) {
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
      let release_url = json
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or("https://github.com/SpikeHD/Orbolay/releases/latest");
      let current_version = format!("v{}", env!("CARGO_PKG_VERSION"));

      if latest_version != current_version {
        app.notify(Notification {
          title: "Update Available!".into(),
          body: format!("Orbolay {} is now available!", latest_version),
          icon: "https://avatars.githubusercontent.com/u/25207995?v=4".to_string(),
          url: Some(release_url.to_string()),
          ..Default::default()
        });
      }
    }
  });
}
