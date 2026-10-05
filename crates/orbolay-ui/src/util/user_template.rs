use std::{
  fs,
  path::{Path, PathBuf},
  sync::{
    LazyLock, Mutex,
    atomic::{AtomicU64, Ordering},
  },
  time::SystemTime,
};

use orbolay_core::{
  config::config_dir,
  dirs::{TEMPLATES_DIR, USER_TEMPLATES},
  user::{User, UserVoiceState},
};
use orbolay_logging::warn;
use scraper::{Html, Selector};

use crate::util::html_template::{
  escape_html, is_network_url, merge_classes, render_tag, theme_decls,
};
use crate::util::theme::Theme;

const ROOT_ID: &str = "user";

const NAME_TOKEN: &str = "{{name}}";
const AVATAR_TOKEN: &str = "{{avatar}}";

const MUTED_ICON_TOKEN: &str = "{{muted-icon}}";
const DEAFENED_ICON_TOKEN: &str = "{{deafened-icon}}";
const STREAMING_ICON_TOKEN: &str = "{{streaming-icon}}";
const CAMERA_ICON_TOKEN: &str = "{{camera-icon}}";

const MUTED_ICON: &str = include_str!("../../../../assets/muted.svg");
const DEAFENED_ICON: &str = include_str!("../../../../assets/deafened.svg");
const STREAMING_ICON: &str = include_str!("../../../../assets/streaming.svg");
const CAMERA_ICON: &str = include_str!("../../../../assets/camera.svg");

const DEFAULT_TEMPLATE: &[u8] = include_bytes!("../../../../templates/user/index.html");

static DEFAULT: LazyLock<UserTemplate> = LazyLock::new(|| {
  let html = std::str::from_utf8(DEFAULT_TEMPLATE).expect("default template is valid UTF-8");
  preprocess(html).expect("the built-in default template must be valid")
});

static GENERATION: AtomicU64 = AtomicU64::new(0);

fn next_generation() -> u64 {
  GENERATION.fetch_add(1, Ordering::Relaxed) + 1
}

struct TemplateCache {
  name: Option<String>,
  mtime: Option<SystemTime>,
  template: UserTemplate,
}

static CACHE: LazyLock<Mutex<TemplateCache>> = LazyLock::new(|| {
  Mutex::new(TemplateCache {
    name: None,
    mtime: None,
    template: default_template(),
  })
});

#[derive(Clone)]
pub struct UserTemplate {
  html: String,
  root_name: String,
  root_attrs: Vec<(String, String)>,
  /// Which version of the template this is
  pub generation: u64,
}

fn default_template() -> UserTemplate {
  DEFAULT.clone()
}

fn user_templates_dir() -> Option<PathBuf> {
  config_dir().map(|dir| dir.join(TEMPLATES_DIR).join(USER_TEMPLATES))
}

fn user_template_path(name: &str) -> Option<PathBuf> {
  user_templates_dir().map(|dir| dir.join(format!("{name}.html")))
}

pub fn list_user_templates() -> Vec<String> {
  let Some(dir) = user_templates_dir() else {
    return Vec::new();
  };
  let Ok(entries) = fs::read_dir(&dir) else {
    return Vec::new();
  };
  let mut names = entries
    .flatten()
    .filter(|entry| {
      let path = entry.path();
      path.is_file() && path.extension().is_some_and(|ext| ext == "html")
    })
    .filter_map(|entry| {
      entry
        .path()
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
    })
    .collect::<Vec<_>>();
  names.sort();
  names
}

fn read_template_file(path: &Path) -> UserTemplate {
  match fs::read_to_string(path) {
    Ok(raw) => match preprocess(&raw) {
      Ok(mut template) => {
        template.generation = next_generation();
        template
      }
      Err(reason) => {
        warn!("Invalid user template {path:?}: {reason}; using the built-in default");
        default_template()
      }
    },
    Err(err) => {
      warn!("Failed to read user template {path:?}: {err}; using the built-in default");
      default_template()
    }
  }
}

pub fn load_user_template(name: Option<&str>) -> UserTemplate {
  let Some(name) = name else {
    return default_template();
  };
  let Some(path) = user_template_path(name) else {
    return default_template();
  };

  let key = fs::metadata(&path).ok().and_then(|m| m.modified().ok());

  let mut cache = CACHE.lock().unwrap();
  if cache.name.as_deref() == Some(name) && cache.mtime == key {
    return cache.template.clone();
  }

  let template = read_template_file(&path);
  cache.name = Some(name.to_string());
  cache.mtime = key;
  cache.template = template.clone();
  template
}

impl UserTemplate {
  pub fn render(
    &self,
    user: &User,
    is_self: bool,
    theme: &Theme,
    is_right_aligned: bool,
  ) -> String {
    let root_tag = render_tag(&self.root_name, &self.root_attrs);
    let mut state_classes = user_template_classes(user, is_self);
    if is_right_aligned {
      state_classes.push("right");
    }
    let mut new_attrs = merge_classes(&self.root_attrs, &state_classes);
    new_attrs.push(("data-user-id".to_string(), user.id.clone()));
    let new_root_tag = render_tag(&self.root_name, &new_attrs);
    let mut html = self.html.replace(&root_tag, &new_root_tag);

    html = html.replace(
      "</head>",
      &format!("<style>:root{{{}}}</style></head>", theme_decls(theme)),
    );
    html = html.replace(AVATAR_TOKEN, &avatar_url(user));
    html = html.replace(NAME_TOKEN, &escape_html(&user.name));
    html = html.replace(
      MUTED_ICON_TOKEN,
      &status_icon(
        user.voice_state == UserVoiceState::Muted,
        "muted",
        MUTED_ICON,
      ),
    );
    html = html.replace(
      DEAFENED_ICON_TOKEN,
      &status_icon(
        user.voice_state == UserVoiceState::Deafened,
        "deafened",
        DEAFENED_ICON,
      ),
    );
    html = html.replace(
      STREAMING_ICON_TOKEN,
      &status_icon(user.streaming, "streaming", STREAMING_ICON),
    );
    html = html.replace(
      CAMERA_ICON_TOKEN,
      &status_icon(user.camera, "camera", CAMERA_ICON),
    );
    html
  }
}

fn status_icon(active: bool, state: &str, svg: &str) -> String {
  if active {
    format!("<span class=\"status-icon {state}\">{svg}</span>")
  } else {
    String::new()
  }
}

fn preprocess(raw: &str) -> Result<UserTemplate, String> {
  let doc = Html::parse_document(raw);
  let root_selector = Selector::parse(&format!("#{ROOT_ID}")).expect("valid selector");

  let roots: Vec<_> = doc.select(&root_selector).collect();
  let root = match roots.as_slice() {
    [root] => *root,
    [] => {
      return Err(format!(
        "must contain exactly one element with id=\"{ROOT_ID}\" (found 0)"
      ));
    }
    _ => {
      return Err(format!(
        "must contain exactly one element with id=\"{ROOT_ID}\" (found {})",
        roots.len()
      ));
    }
  };

  if doc
    .select(&Selector::parse("script").expect("valid selector"))
    .count()
    > 0
  {
    return Err("JavaScript is not supported: remove the <script> element".into());
  }

  for link in doc.select(&Selector::parse("link[href]").expect("valid selector")) {
    let href = link.attr("href").expect("selector matched on [href]");
    if !is_network_url(href) {
      return Err(format!(
        "stylesheet <link> href must be a network URL (http/https), got: {href}"
      ));
    }
  }

  for img in doc.select(&Selector::parse("img[src]").expect("valid selector")) {
    let src = img.attr("src").expect("selector matched on [src]");
    if src == AVATAR_TOKEN || is_network_url(src) {
      continue;
    }
    return Err(format!(
      "<img> src must be a network URL or {AVATAR_TOKEN}, got: {src}"
    ));
  }

  if !raw.contains(NAME_TOKEN) {
    return Err(format!("must contain the {NAME_TOKEN} token"));
  }

  let element = root.value();
  Ok(UserTemplate {
    html: doc.html(),
    root_name: element.name().to_string(),
    root_attrs: element
      .attrs()
      .map(|(name, value)| (name.to_string(), value.to_string()))
      .collect(),
    generation: 0,
  })
}

pub fn user_template_classes(user: &User, is_self: bool) -> Vec<&'static str> {
  let mut classes = Vec::new();

  match user.voice_state {
    UserVoiceState::Speaking => classes.push("speaking"),
    UserVoiceState::Muted => classes.push("muted"),
    UserVoiceState::Deafened => classes.push("deafened"),
    UserVoiceState::NotSpeaking => {}
  }

  if user.streaming {
    classes.push("streaming");
  }

  if user.camera {
    classes.push("camera");
  }

  if user.avatar.is_empty() {
    classes.push("no-avatar");
  }

  if is_self {
    classes.push("self");
  }

  classes
}

fn avatar_url(user: &User) -> String {
  if user.avatar.is_empty() {
    return String::new();
  }

  format!(
    "https://cdn.discordapp.com/avatars/{}/{}.png?size=160",
    user.id, user.avatar
  )
}
