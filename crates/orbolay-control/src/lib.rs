use std::io::{BufRead, BufReader, Write};

#[cfg(unix)]
use interprocess::local_socket::{GenericFilePath, ToFsName};
#[cfg(windows)]
use interprocess::local_socket::{GenericNamespaced, ToNsName};
use interprocess::{
  TryClone,
  local_socket::{ListenerOptions, prelude::*},
};
use orbolay_logging::{error, log, warn};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Command {
  ToggleOverlay,
  Mute,
  Deafen,
}

#[cfg(unix)]
pub fn socket_path() -> Option<std::path::PathBuf> {
  Some(dirs::runtime_dir()?.join("orbolay").join("orbolay.sock"))
}

#[cfg(windows)]
pub const SOCKET_NAME: &str = "orbolay-control";

pub fn start(handler: impl Fn(Command) + Send + Sync + 'static) {
  let handler = std::sync::Arc::new(handler);

  std::thread::spawn(move || {
    let Ok(listener) = make_listener() else {
      error!("Failed to create control listener");
      return;
    };

    log!("Control socket listening");

    loop {
      match listener.accept() {
        Ok(stream) => {
          let handler = handler.clone();
          std::thread::spawn(move || {
            serve(stream, handler.as_ref());
            log!("Control socket client disconnected");
          });
        }
        Err(e) => {
          error!("Control listener error: {}", e);
          break;
        }
      }
    }
  });
}

fn make_listener() -> Result<LocalSocketListener, Box<dyn std::error::Error>> {
  #[cfg(unix)]
  {
    let path = socket_path().ok_or("Could not determine socket path")?;
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    let _ = std::fs::remove_file(&path);
    let name = path.to_fs_name::<GenericFilePath>()?;
    Ok(ListenerOptions::new().name(name).create_sync()?)
  }

  #[cfg(windows)]
  {
    let name = SOCKET_NAME.to_ns_name::<GenericNamespaced>()?;
    Ok(ListenerOptions::new().name(name).create_sync()?)
  }
}

fn serve(stream: LocalSocketStream, handler: &impl Fn(Command)) {
  let mut line = String::new();
  let mut reader = BufReader::new(stream);

  while reader.read_line(&mut line).unwrap_or(0) > 0 {
    let cmd_line = line.trim().to_owned();

    if cmd_line.is_empty() {
      continue;
    }

    line.clear();

    match serde_json::from_str::<Command>(&cmd_line) {
      Ok(cmd) => {
        log!("Received control command: {:?}", cmd);
        handler(cmd);
      }
      Err(e) => warn!("Invalid control command: {}", e),
    }
  }
}

pub struct Client {
  pub stream: LocalSocketStream,
}

impl Client {
  pub fn connect() -> Result<Self, Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
      let path = socket_path().ok_or("Could not determine socket path")?;
      let stream = LocalSocketStream::connect(path.to_fs_name::<GenericFilePath>()?)?;
      Ok(Self { stream })
    }

    #[cfg(windows)]
    {
      let stream = LocalSocketStream::connect(SOCKET_NAME.to_ns_name::<GenericNamespaced>()?)?;
      Ok(Self { stream })
    }
  }

  pub fn send(&mut self, cmd: &Command) -> Result<(), Box<dyn std::error::Error>> {
    let mut payload = serde_json::to_string(cmd)?;
    payload.push('\n');
    self.stream.write_all(payload.as_bytes())?;
    Ok(())
  }

  pub fn fork(&self) -> Result<Self, Box<dyn std::error::Error>> {
    Ok(Self {
      stream: self.stream.try_clone()?,
    })
  }
}
