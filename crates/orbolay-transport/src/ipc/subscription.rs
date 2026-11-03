use orbolay_logging::log;
use serde_json::Value;

use crate::ipc::{OP_FRAME, SharedStream, ipc_write};

pub fn subscribe(
  stream: &SharedStream,
  event: &str,
  data: Option<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
  let subscribe_msg = serde_json::json!({
    "cmd": "SUBSCRIBE",
    "evt": event,
    "args": data.unwrap_or_else(|| serde_json::json!({})),
    "nonce": event,
  });
  ipc_write(stream, OP_FRAME, &subscribe_msg.to_string())?;
  Ok(())
}

pub fn unsubscribe(
  stream: &SharedStream,
  event: &str,
  data: Option<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
  let unsubscribe_msg = serde_json::json!({
    "cmd": "UNSUBSCRIBE",
    "evt": event,
    "args": data.unwrap_or_else(|| serde_json::json!({})),
    "nonce": event,
  });
  ipc_write(stream, OP_FRAME, &unsubscribe_msg.to_string())?;
  Ok(())
}

fn subscribe_channel(
  stream: &SharedStream,
  event: &str,
  channel_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let data = serde_json::json!({ "channel_id": channel_id });
  subscribe(stream, event, Some(data))
}

fn unsubscribe_channel(
  stream: &SharedStream,
  event: &str,
  channel_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let data = serde_json::json!({ "channel_id": channel_id });
  unsubscribe(stream, event, Some(data))
}

pub fn subscribe_voice_channel(
  stream: &SharedStream,
  channel_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  subscribe_channel(stream, "VOICE_STATE_CREATE", channel_id)?;
  subscribe_channel(stream, "VOICE_STATE_UPDATE", channel_id)?;
  subscribe_channel(stream, "VOICE_STATE_DELETE", channel_id)?;
  subscribe_channel(stream, "SCREENSHARE_STATE_UPDATE", channel_id)?;
  subscribe_channel(stream, "SPEAKING_START", channel_id)?;
  subscribe_channel(stream, "SPEAKING_STOP", channel_id)?;
  Ok(())
}

pub fn subscribe_voice_global(stream: &SharedStream) -> Result<(), Box<dyn std::error::Error>> {
  subscribe(stream, "VOICE_CHANNEL_SELECT", None)?;
  subscribe(stream, "VOICE_SETTINGS_UPDATE", None)?;
  subscribe(stream, "VOICE_CONNECTION_STATUS", None)?;
  subscribe(stream, "NOTIFICATION_CREATE", None)?;
  Ok(())
}

pub fn unsubscribe_voice_channel(
  stream: &SharedStream,
  channel_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  unsubscribe_channel(stream, "VOICE_STATE_CREATE", channel_id)?;
  unsubscribe_channel(stream, "VOICE_STATE_UPDATE", channel_id)?;
  unsubscribe_channel(stream, "VOICE_STATE_DELETE", channel_id)?;
  unsubscribe_channel(stream, "SCREENSHARE_STATE_UPDATE", channel_id)?;
  unsubscribe_channel(stream, "SPEAKING_START", channel_id)?;
  unsubscribe_channel(stream, "SPEAKING_STOP", channel_id)?;
  Ok(())
}

pub fn sync_voice_chat(
  stream: &SharedStream,
  subscribed: &mut Option<String>,
  want: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
  if *subscribed == want {
    return Ok(());
  }

  if let Some(old) = std::mem::replace(subscribed, want.clone()) {
    unsubscribe_channel(stream, "MESSAGE_CREATE", &old)?;
    log!("Voice channel chat: unsubscribed");
  }

  if let Some(new) = want {
    subscribe_channel(stream, "MESSAGE_CREATE", &new)?;
    log!("Voice channel chat: subscribed");
  }

  Ok(())
}
