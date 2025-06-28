use sysinfo::{ProcessRefreshKind, RefreshKind, System};

pub mod image;

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

// Check if there is already an orbolay process running
pub fn is_already_running() -> bool {
  let sys = System::new_with_specifics(
    RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
  );
  let procs = sys.processes();

  for proc in procs.values() {
    if proc
      .name()
      .to_ascii_lowercase()
      .to_str()
      .unwrap_or("")
      .contains("orbolay")
    {
      return true;
    }
  }

  false
}
