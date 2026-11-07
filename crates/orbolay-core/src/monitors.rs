use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorInfo {
  pub id: String,
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

pub fn pick_display(
  count: usize,
  name_of: impl Fn(usize) -> Option<String>,
  primary: Option<usize>,
  display_name: Option<&str>,
  display_idx: Option<usize>,
) -> Option<usize> {
  let is_named =
    |idx: &usize| display_name.is_none_or(|name| name_of(*idx).as_deref() == Some(name));
  let by_idx = display_idx.filter(|idx| *idx < count);

  by_idx
    .filter(is_named)
    .or_else(|| display_name.and((0..count).find(is_named)))
    .or(by_idx)
    .or(primary)
}
