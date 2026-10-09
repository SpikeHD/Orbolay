use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
  pub name: String,
  pub is_primary: bool,
  pub width: u32,
  pub height: u32,
}

static CACHE: Mutex<Vec<MonitorInfo>> = Mutex::new(Vec::new());

pub fn set_monitors(infos: Vec<MonitorInfo>) {
  *CACHE.lock().unwrap() = infos;
}

pub fn monitors_list() -> Vec<MonitorInfo> {
  CACHE.lock().unwrap().clone()
}
