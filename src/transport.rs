use crate::{
  Args, app_state::AppHandle, config::TransportMode, error, ipc, util::bridge::BridgeMessage,
  websocket,
};

pub fn create_transport_thread(
  app: AppHandle,
  args: Args,
  ws_recv: flume::Receiver<BridgeMessage>,
) {
  std::thread::spawn(move || {
    let ws_port = args.port;
    let transport_mode = app.read(|state| state.config.transport_mode.clone());

    if args.websocket || (transport_mode == TransportMode::Websocket && !args.ipc) {
      app.update(|state| state.transport_mode = TransportMode::Websocket);
      if let Err(e) = websocket::create_websocket(ws_port, app.clone(), ws_recv) {
        error!("Websocket server failed on port {}: {}", ws_port, e);
      }
      return;
    }

    if args.ipc || (transport_mode == TransportMode::Ipc && !args.websocket) {
      app.update(|state| state.transport_mode = TransportMode::Ipc);
      if let Err(e) = ipc::create_ipc_connection(app.clone(), ws_recv) {
        error!("IPC connection failed: {}", e);
      }
      return;
    }

    error!("No valid transport mode selected.");
  });
}
