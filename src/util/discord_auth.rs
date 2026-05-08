use serde_json::json;
use url::Url;

use crate::log;

pub fn build_rpc_authorize_request(client_id: impl Into<String>) -> serde_json::Value {
  json!({
    "cmd": "AUTHORIZE",
    "args": {
      "client_id": client_id.into(),
      "scopes": ["rpc", "messages.read", "rpc.notifications.read"],
      "prompt": "none"
    },
    "nonce": "helloworld"
  })
}

pub fn build_rpc_authenticate_request(access_token: impl Into<String>) -> serde_json::Value {
  json!({
    "cmd": "AUTHENTICATE",
    "args": {
      "access_token": access_token.into()
    },
    "nonce": "helloworld"
  })
}

pub fn extract_auth_code(code: &str) -> Option<String> {
  const ATTEMPTS: u8 = 3;
  let url = Url::parse("https://streamkit.discord.com/overlay/token").ok()?;
  let body = json!({
    "code": code,
  });

  for attempt in 1..ATTEMPTS + 1 {
    let mut response = ureq::post(url.as_str())
      .header("Content-Type", "application/json")
      .send(&body.to_string())
      .ok()?;
    let body = response.body_mut();
    let body = body.read_to_string().ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&body).ok()?;
    if let Some(token) = parsed.get("access_token").and_then(|t| t.as_str()) {
      return Some(token.to_string());
    }

    log!(
      "Failed to extract access token from StreamKit response, attempt {}/{}",
      attempt,
      ATTEMPTS
    );
  }

  log!(
    "Failed to extract access token from StreamKit response after {} attempts",
    ATTEMPTS
  );

  None
}
