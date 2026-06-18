use std::process::Command;

pub fn launch_target(cmd: Vec<String>) {
  std::thread::spawn(move || {
    let args = cmd.get(1..);
    let mut proc = Command::new(cmd[0].clone());

    if let Some(args) = args {
        proc.args(args);
    }

    let _ = proc.status();

    std::process::exit(0);
  });
}
