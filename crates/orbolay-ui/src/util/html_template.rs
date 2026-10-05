use crate::util::theme::{Theme, to_hex};

pub fn escape_html(input: &str) -> String {
  input
    .replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#39;")
}

pub fn escape_attr(value: &str) -> String {
  let mut out = String::with_capacity(value.len());
  for c in value.chars() {
    match c {
      '&' => out.push_str("&amp;"),
      '\u{00A0}' => out.push_str("&nbsp;"),
      '"' => out.push_str("&quot;"),
      '<' => out.push_str("&lt;"),
      '>' => out.push_str("&gt;"),
      c => out.push(c),
    }
  }
  out
}

pub fn render_tag(name: &str, attrs: &[(String, String)]) -> String {
  let mut tag = String::with_capacity(64);
  tag.push('<');
  tag.push_str(name);
  for (name, value) in attrs {
    tag.push_str(&format!(" {name}=\"{}\"", escape_attr(value)));
  }
  tag.push('>');
  tag
}

pub fn merge_classes(attrs: &[(String, String)], extra: &[&str]) -> Vec<(String, String)> {
  let existing = attrs
    .iter()
    .find(|(name, _)| name == "class")
    .map(|(_, value)| value.clone());

  let mut classes = existing
    .as_deref()
    .map(|value| {
      value
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>()
    })
    .unwrap_or_default();
  for class in extra {
    if !classes.iter().any(|existing| existing == class) {
      classes.push((*class).to_string());
    }
  }

  if classes.is_empty() && existing.is_none() {
    return attrs.to_vec();
  }

  let joined = classes.join(" ");
  if existing.is_some() {
    attrs
      .iter()
      .map(|(name, value)| {
        if name == "class" {
          (name.clone(), joined.clone())
        } else {
          (name.clone(), value.clone())
        }
      })
      .collect()
  } else {
    let mut out = attrs.to_vec();
    out.push(("class".into(), joined));
    out
  }
}

pub fn is_network_url(url: &str) -> bool {
  url.starts_with("http://") || url.starts_with("https://")
}

pub fn theme_decls(theme: &Theme) -> String {
  format!(
    "--gray: {}; --darkish-gray: {}; --light-gray: {}; --superlight-gray: {}; --muted-gray: {}; --text: {}; --border-radius: {}px;",
    to_hex(theme.gray),
    to_hex(theme.darkish_gray),
    to_hex(theme.light_gray),
    to_hex(theme.superlight_gray),
    to_hex(theme.muted_gray),
    to_hex(theme.text_color),
    theme.border_radius
  )
}
