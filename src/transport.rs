use crate::{Args, app_state::SharedAppState, error, ipc, util::bridge::BridgeMessage, warn, websocket};

pub fn create_transport_thread(
  shared: SharedAppState,
  redraw_tx: flume::Sender<()>,
  args: Args,
  ws_recv: flume::Receiver<BridgeMessage>,
) {
  std::thread::spawn(move || {
    let ws_port = args.port;

    if args.websocket {
      if let Err(e) = websocket::create_websocket(ws_port, shared, redraw_tx, ws_recv) {
        error!("Websocket server failed on port {}: {}", ws_port, e);
      }
      return;
    }

    let ipc_receiver = ws_recv.clone();
    let redraw_ipc = redraw_tx.clone();
    if let Err(e) = ipc::create_ipc_connection(shared.clone(), redraw_ipc, ipc_receiver) {
      warn!("IPC connection failed: {}", e);
      if let Err(e) = websocket::create_websocket(ws_port, shared, redraw_tx, ws_recv) {
        error!("Websocket server failed on port {}: {}", ws_port, e);
      }
    }
  });
}
