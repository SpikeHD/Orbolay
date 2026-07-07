use orbolay_core::{app_state::AppHandle, config::TransportMode, util::bridge::BridgeMessage};
use orbolay_logging::error;

use crate::{ipc, websocket};

pub fn create_transport_thread(
  app: AppHandle,
  port: u16,
  force: Option<TransportMode>,
  transport_recv: flume::Receiver<BridgeMessage>,
) {
  std::thread::spawn(move || {
    let ws_port = port;
    let transport_mode = force.unwrap_or_else(|| app.read(|state| state.config.transport_mode.clone()));

    match transport_mode {
      TransportMode::Websocket => {
        app.update(|state| state.transport_mode = TransportMode::Websocket);
        if let Err(e) = websocket::create_websocket(ws_port, app.clone(), transport_recv) {
          error!("Websocket server failed on port {}: {}", ws_port, e);
        }
      }
      TransportMode::Ipc => {
        app.update(|state| state.transport_mode = TransportMode::Ipc);
        if let Err(e) = ipc::create_ipc_connection(app.clone(), transport_recv) {
          error!("IPC connection failed: {}", e);
        }
      }
    }
  });
}
