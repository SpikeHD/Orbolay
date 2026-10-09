use std::{collections::HashSet, path::Path};

use sysinfo::{Pid, ProcessRefreshKind, RefreshKind, System};

// Collect our own PID and the full chain of ancestor PIDs.
fn self_and_ancestor_pids() -> HashSet<Pid> {
  let sys = System::new_with_specifics(
    RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
  );
  let procs = sys.processes();

  let mut pids = HashSet::new();
  let mut current = Pid::from_u32(std::process::id());

  for _ in 0..64 {
    if !pids.insert(current) {
      break;
    }
    let Some(pid) = procs.get(&current).and_then(|proc| proc.parent()) else {
      break;
    };
    current = pid;
  }

  pids
}

// The basename of an on-disk executable but lowercase
fn exe_basename(exe: &Path) -> String {
  exe
    .file_name()
    .map(|n| n.to_string_lossy().to_ascii_lowercase())
    .unwrap_or_default()
}

// Check if there is already an orbolay process running
pub fn is_already_running() -> bool {
  let ours = self_and_ancestor_pids();

  let our_name = std::env::current_exe()
    .ok()
    .map(|exe| exe_basename(&exe))
    .unwrap_or_else(|| "orbolay".to_string());

  let sys = System::new_with_specifics(
    RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
  );
  let procs = sys.processes();

  for proc in procs.values() {
    if ours.contains(&proc.pid()) {
      continue;
    }

    let Some(exe) = proc.exe() else {
      continue;
    };
    if exe_basename(exe) == our_name {
      return true;
    }
  }

  false
}
