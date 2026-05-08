mod commands;
mod payloads;
mod subscription;

pub mod ipc_message_handler;
pub mod setters;
pub mod ui_message_handler;

pub use commands::create_ipc_connection;
pub use ipc_message_handler::handle_ipc_message;
pub use payloads::*;

pub use subscription::{
  subscribe_voice_channel, subscribe_voice_global, unsubscribe_voice_channel,
};
pub use ui_message_handler::handle_ui_message;

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

// IPC opcodes
pub const OP_HANDSHAKE: u32 = 0;
pub const OP_FRAME: u32 = 1;
pub const OP_CLOSE: u32 = 2;

pub fn ipc_write(
  stream: &mut UnixStream,
  opcode: u32,
  payload: &str,
) -> Result<(), Box<dyn std::error::Error>> {
  let payload_bytes = payload.as_bytes();
  let len = payload_bytes.len() as u32;
  let mut header = [0u8; 8];
  header[0..4].copy_from_slice(&opcode.to_le_bytes());
  header[4..8].copy_from_slice(&len.to_le_bytes());
  stream.write_all(&header)?;
  stream.write_all(payload_bytes)?;
  Ok(())
}

pub fn ipc_read(stream: &mut UnixStream) -> Result<(u32, String), std::io::Error> {
  let mut header = [0u8; 8];
  stream.read_exact(&mut header)?;
  let opcode = u32::from_le_bytes(header[0..4].try_into().unwrap());
  let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
  let mut payload = vec![0u8; len];
  stream.read_exact(&mut payload)?;
  let s = String::from_utf8(payload)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
  Ok((opcode, s))
}
