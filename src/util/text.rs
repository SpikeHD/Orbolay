pub fn truncate(text: impl AsRef<str>, max: usize) -> String {
  let text = text.as_ref();
  if text.len() > max {
    format!("{}...", &text[..max])
  } else {
    text.to_string()
  }
}

// Remove some stuff that would break formatting
pub fn strip(text: impl AsRef<str>) -> String {
  text.as_ref().replace("\n", " ")
}

pub fn censor(text: impl AsRef<str>) -> String {
  format!("{}...", text.as_ref().chars().next().unwrap_or('?'))
}