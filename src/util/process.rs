use std::collections::HashSet;

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

// Check if there is already an orbolay process running
pub fn is_already_running() -> bool {
  let ours = self_and_ancestor_pids();

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
      && !ours.contains(&proc.pid())
    {
      return true;
    }
  }

  false
}
