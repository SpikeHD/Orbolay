use crate::config::config_dir;

pub const TEMPLATES_DIR: &str = "templates";
pub const USER_TEMPLATES: &str = "users";
pub const NOTIFICATION_TEMPLATES: &str = "notifications";

pub fn ensure_config_dirs() {
  let Some(dir) = config_dir() else { return };

  let templates_dir = dir.join(TEMPLATES_DIR);
  let users_dir = templates_dir.join(USER_TEMPLATES);
  let notifications_dir = templates_dir.join(NOTIFICATION_TEMPLATES);

  for sub in [&dir, &templates_dir, &users_dir, &notifications_dir] {
    if !sub.exists() {
      let _ = std::fs::create_dir_all(sub);
    }
  }
}
