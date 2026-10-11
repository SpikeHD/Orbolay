use std::process::Command;

pub fn launch_target(cmd: Vec<String>, wayland_display: Result<String, std::env::VarError>) {
  std::thread::spawn(move || {
    let args = cmd.get(1..);
    let mut proc = Command::new(cmd[0].clone());

    if let Some(args) = args {
      proc.args(args);
    }

    match wayland_display {
      Ok(val) => {
        proc.env("WAYLAND_DISPLAY", val);
      }
      Err(_) => {
        proc.env_remove("WAYLAND_DISPLAY");
      }
    }

    let _ = proc.status();

    std::process::exit(0);
  });
}
