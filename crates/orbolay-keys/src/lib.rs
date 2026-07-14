use std::cell::Cell;

thread_local! {
  pub static DEFAULT_OVERLAY_TOGGLE: Cell<[&str; 2]> =
    const { Cell::new(["ControlLeft", "BackQuote"]) };
}

#[cfg(not(target_os = "macos"))]
use rdev::Key;

#[cfg(not(target_os = "macos"))]
pub fn string_to_key(string: impl AsRef<str>) -> Option<Key> {
  let s = string.as_ref();
  serde_json::from_str::<Key>(s)
    .or_else(|_| serde_json::from_str::<Key>(&format!("\"{}\"", s)))
    .ok()
}

#[cfg(not(target_os = "macos"))]
pub fn strings_to_keys(strings: Vec<impl AsRef<str>>) -> Vec<Key> {
  strings.iter().filter_map(string_to_key).collect()
}

#[cfg(not(target_os = "macos"))]
pub fn key_to_string(key: &Key) -> String {
  serde_json::to_string(key)
    .unwrap_or_default()
    .trim_matches('"')
    .to_owned()
}

#[cfg(not(target_os = "macos"))]
pub fn keys_to_strings(keys: Vec<Key>) -> Vec<String> {
  keys.iter().map(key_to_string).collect()
}
