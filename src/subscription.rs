use std::os::unix::net::UnixStream;

use serde_json::Value;

use crate::ipc::{OP_FRAME, ipc_write};

pub fn subscribe(
  mut stream: &mut UnixStream,
  event: &str,
  data: Option<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
  let subscribe_msg = serde_json::json!({
    "cmd": "SUBSCRIBE",
    "evt": event,
    "args": data.unwrap_or_default(),
    "nonce": event,
  });
  ipc_write(&mut stream, OP_FRAME, &subscribe_msg.to_string())?;
  Ok(())
}

pub fn unsubscribe(
  mut stream: &mut UnixStream,
  event: &str,
  data: Option<Value>,
) -> Result<(), Box<dyn std::error::Error>> {
  let unsubscribe_msg = serde_json::json!({
    "cmd": "UNSUBSCRIBE",
    "evt": event,
    "args": data.unwrap_or_default(),
    "nonce": event,
  });
  ipc_write(&mut stream, OP_FRAME, &unsubscribe_msg.to_string())?;
  Ok(())
}

fn subscribe_channel(
  stream: &mut UnixStream,
  event: &str,
  channel_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let data = serde_json::json!({ "channel_id": channel_id });
  subscribe(stream, event, Some(data))
}

fn unsubscribe_channel(
  stream: &mut UnixStream,
  event: &str,
  channel_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let data = serde_json::json!({ "channel_id": channel_id });
  unsubscribe(stream, event, Some(data))
}

pub fn subscribe_voice_channel(
  stream: &mut UnixStream,
  channel_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  subscribe_channel(stream, "VOICE_STATE_CREATE", channel_id)?;
  subscribe_channel(stream, "VOICE_STATE_UPDATE", channel_id)?;
  subscribe_channel(stream, "VOICE_STATE_DELETE", channel_id)?;
  subscribe_channel(stream, "SPEAKING_START", channel_id)?;
  subscribe_channel(stream, "SPEAKING_STOP", channel_id)?;
  Ok(())
}

pub fn unsubscribe_voice_channel(
  stream: &mut UnixStream,
  channel_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  unsubscribe_channel(stream, "VOICE_STATE_CREATE", channel_id)?;
  unsubscribe_channel(stream, "VOICE_STATE_UPDATE", channel_id)?;
  unsubscribe_channel(stream, "VOICE_STATE_DELETE", channel_id)?;
  unsubscribe_channel(stream, "SPEAKING_START", channel_id)?;
  unsubscribe_channel(stream, "SPEAKING_STOP", channel_id)?;
  Ok(())
}
