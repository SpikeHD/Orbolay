use dioxus::signals::{Signal, SyncStorage};

use crate::{Args, app_state::AppState, error, ipc, util::bridge::BridgeMessage, warn, websocket};

pub fn create_transport_thread(
  app_state: Signal<AppState, SyncStorage>,
  args: Args,
  ws_recv: flume::Receiver<BridgeMessage>,
) {
  std::thread::spawn(move || {
    let ws_port = args.port;

    if args.websocket {
      if let Err(e) = websocket::create_websocket(ws_port, app_state, ws_recv) {
        error!("Websocket server failed on port {}: {}", ws_port, e);
      }
      return;
    }

    let ipc_receiver = ws_recv.clone();
    if let Err(e) = ipc::create_ipc_connection(app_state, ipc_receiver) {
      warn!("IPC connection failed: {}", e);
      if let Err(e) = websocket::create_websocket(ws_port, app_state, ws_recv) {
        error!("Websocket server failed on port {}: {}", ws_port, e);
      }
    }
  });
}
