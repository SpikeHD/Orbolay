use crate::{
  Args, app_state::SharedAppState, config::TransportMode, error, ipc, util::bridge::BridgeMessage,
  websocket,
};

pub fn create_transport_thread(
  shared: SharedAppState,
  redraw_tx: flume::Sender<()>,
  args: Args,
  ws_recv: flume::Receiver<BridgeMessage>,
) {
  std::thread::spawn(move || {
    let ws_port = args.port;
    let transport_mode = shared.read().unwrap().config.transport_mode.clone();

    if args.websocket || (transport_mode == TransportMode::Websocket && !args.ipc) {
      if let Err(e) = websocket::create_websocket(ws_port, shared, redraw_tx, ws_recv) {
        error!("Websocket server failed on port {}: {}", ws_port, e);
      }
      return;
    }

    if args.ipc || (transport_mode == TransportMode::Ipc && !args.websocket) {
      if let Err(e) = ipc::create_ipc_connection(shared, redraw_tx, ws_recv) {
        error!("IPC connection failed: {}", e);
      }
      return;
    }

    error!("No valid transport mode selected.");
  });
}
