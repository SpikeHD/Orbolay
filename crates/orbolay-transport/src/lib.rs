pub mod config_watcher;
pub mod ipc;
pub mod transport;
pub mod updates;
pub mod util;
pub mod websocket;

pub const CLIENT_ID: &str = "207646673902501888";

pub use config_watcher::start_config_watcher;
pub use transport::create_transport_thread;
pub use updates::maybe_notify_update;
