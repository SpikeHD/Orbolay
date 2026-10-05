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
  payloads::{Notification, NotificationKind},
  util::text::strip,
};
use orbolay_logging::warn;
use scraper::{Html, Selector};

use crate::util::html_template::{escape_html, is_network_url, merge_classes, render_tag, theme_decls};
use crate::util::theme::Theme;

pub const TEMPLATES_DIR: &str = "templates";
pub const NOTIFICATION_TEMPLATES: &str = "notifications";

pub const ACTION_ATTR: &str = "data-action";

const ROOT_ID: &str = "notification";

const TITLE_TOKEN: &str = "{{title}}";
const BODY_TOKEN: &str = "{{body}}";
const ICON_TOKEN: &str = "{{icon}}";
const ACTIONS_TOKEN: &str = "{{actions}}";

const DEFAULT_TEMPLATE: &[u8] =
  include_bytes!("../../../../templates/notification/index.html");

static DEFAULT: LazyLock<NotificationTemplate> = LazyLock::new(|| {
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
  template: NotificationTemplate,
}

static CACHE: LazyLock<Mutex<TemplateCache>> = LazyLock::new(|| {
  Mutex::new(TemplateCache {
    name: None,
    mtime: None,
    template: default_template(),
  })
});

#[derive(Clone)]
pub struct NotificationTemplate {
  html: String,
  root_name: String,
  root_attrs: Vec<(String, String)>,
  /// Which version of the template this is
  pub generation: u64,
}

fn default_template() -> NotificationTemplate {
  DEFAULT.clone()
}

fn notification_templates_dir() -> Option<PathBuf> {
  config_dir().map(|dir| dir.join(TEMPLATES_DIR).join(NOTIFICATION_TEMPLATES))
}

fn notification_template_path(name: &str) -> Option<PathBuf> {
  notification_templates_dir().map(|dir| dir.join(format!("{name}.html")))
}

pub fn list_notification_templates() -> Vec<String> {
  let Some(dir) = notification_templates_dir() else {
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

fn read_template_file(path: &Path) -> NotificationTemplate {
  match fs::read_to_string(path) {
    Ok(raw) => match preprocess(&raw) {
      Ok(mut template) => {
        template.generation = next_generation();
        template
      }
      Err(reason) => {
        warn!("Invalid notification template {path:?}: {reason}; using the built-in default");
        default_template()
      }
    },
    Err(err) => {
      warn!("Failed to read notification template {path:?}: {err}; using the built-in default");
      default_template()
    }
  }
}

pub fn load_notification_template(name: Option<&str>) -> NotificationTemplate {
  let Some(name) = name else {
    return default_template();
  };
  let Some(path) = notification_template_path(name) else {
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

impl NotificationTemplate {
  pub fn render(&self, notification: &Notification, theme: &Theme) -> String {
    let root_tag = render_tag(&self.root_name, &self.root_attrs);
    let mut classes = Vec::new();
    if notification.icon.is_empty() {
      classes.push("no-icon");
    }
    if notification
      .actions
      .as_deref()
      .is_some_and(|actions| !actions.is_empty())
    {
      classes.push("has-actions");
    }
    let new_attrs = merge_classes(&self.root_attrs, &classes);
    let new_root_tag = render_tag(&self.root_name, &new_attrs);
    let mut html = self.html.replace(&root_tag, &new_root_tag);

    html = html.replace(
      "</head>",
      &format!("<style>:root{{{}}}</style></head>", theme_decls(theme)),
    );
    html = html.replace(TITLE_TOKEN, &escape_html(&notification.title));
    html = html.replace(BODY_TOKEN, &escape_html(&strip(&notification.body)));
    html = html.replace(ICON_TOKEN, &notification.icon);
    html = html.replace(ACTIONS_TOKEN, &actions_html(notification));
    html
  }
}

fn actions_html(notification: &Notification) -> String {
  match &notification.actions {
    Some(actions) => actions
      .iter()
      .enumerate()
      .map(|(index, action)| {
        let kind_class = if action.kind == NotificationKind::Secondary {
          "action secondary"
        } else {
          "action"
        };
        format!(
          "<button class=\"{kind_class}\" {ACTION_ATTR}=\"{index}\">{}</button>",
          escape_html(&action.label)
        )
      })
      .collect::<Vec<_>>()
      .join(""),
    None => String::new(),
  }
}

fn preprocess(raw: &str) -> Result<NotificationTemplate, String> {
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
    if src == ICON_TOKEN || is_network_url(src) {
      continue;
    }
    return Err(format!(
      "<img> src must be a network URL or {ICON_TOKEN}, got: {src}"
    ));
  }

  for token in [TITLE_TOKEN, BODY_TOKEN] {
    if !raw.contains(token) {
      return Err(format!("must contain the {token} token"));
    }
  }

  let element = root.value();
  Ok(NotificationTemplate {
    html: doc.html(),
    root_name: element.name().to_string(),
    root_attrs: element
      .attrs()
      .map(|(name, value)| (name.to_string(), value.to_string()))
      .collect(),
    generation: 0,
  })
}
