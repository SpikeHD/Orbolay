use std::collections::HashMap;
use std::process::Command;

pub fn launch_target(cmd: Vec<String>, extra_env: Option<HashMap<String, Option<String>>>) {
  std::thread::spawn(move || {
    let args = cmd.get(1..);
    let mut proc = Command::new(cmd[0].clone());

    if let Some(args) = args {
      proc.args(args);
    }

    for (key, value) in extra_env.unwrap_or_default() {
      match value {
        Some(val) => proc.env(key, val),
        None => proc.env_remove(&key),
      };
    }

    let _ = proc.status();

    std::process::exit(0);
  });
}
