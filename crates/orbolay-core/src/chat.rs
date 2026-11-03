use serde::Deserialize;

use crate::{app_state::AppState, payloads::Notification, user::avatar_url};

pub const MAX_CHAT_ROWS: usize = 3;

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessagePayload {
  pub channel_id: String,
  #[serde(default)]
  pub channel_name: Option<String>,
  pub message: ChatMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMessage {
  pub id: String,
  #[serde(rename = "type", default)]
  pub kind: Option<u64>,
  #[serde(default)]
  pub content: Option<String>,
  #[serde(default)]
  pub nick: Option<String>,
  #[serde(default)]
  pub blocked: Option<bool>,
  #[serde(default)]
  pub guild_id: Option<String>,
  pub author: ChatAuthor,
  #[serde(default)]
  pub mentions: Vec<ChatMention>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatAuthor {
  pub id: String,
  #[serde(default)]
  pub username: Option<String>,
  #[serde(default)]
  pub global_name: Option<String>,
  #[serde(default)]
  pub avatar: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatMention {
  pub id: String,
  #[serde(default)]
  pub username: Option<String>,
  #[serde(default)]
  pub global_name: Option<String>,
}

impl ChatMessage {
  fn display_name(&self) -> String {
    [&self.nick, &self.author.global_name, &self.author.username]
      .into_iter()
      .flatten()
      .find(|name| !name.is_empty())
      .cloned()
      .unwrap_or_default()
  }
}

impl AppState {
  pub fn receive_chat(&mut self, payload: &ChatMessagePayload) -> bool {
    let Some(row) = chat_row(self, payload) else {
      return false;
    };

    self.push_chat(row);
    true
  }
}

fn chat_row(state: &AppState, payload: &ChatMessagePayload) -> Option<Notification> {
  let message = &payload.message;
  let guild_id = message.guild_id.clone()?;
  let already_shown = state
    .messages
    .iter()
    .chain(&state.chat_messages)
    .any(|row| row.message_id.as_deref() == Some(message.id.as_str()));

  if !state.config.voice_chat_enabled()
    || state.current_channel.is_empty()
    || payload.channel_id != state.current_channel
    || message.author.id == state.user_id
    || !matches!(message.kind.unwrap_or(0), 0 | 19)
    || message.blocked.unwrap_or(false)
    || already_shown
  {
    return None;
  }

  let body = format_content(
    message.content.as_deref().unwrap_or_default(),
    &message.mentions,
  );
  if body.trim().is_empty() {
    return None;
  }

  Some(Notification {
    title: message.display_name(),
    body,
    icon: avatar_url(
      &message.author.id,
      message.author.avatar.as_deref().unwrap_or_default(),
    ),
    guild_id: Some(guild_id),
    channel_id: Some(payload.channel_id.clone()),
    message_id: Some(message.id.clone()),
    channel_name: payload
      .channel_name
      .clone()
      .filter(|name| !name.is_empty())
      .or_else(|| state.channel_names.get(&payload.channel_id).cloned()),
    ..Default::default()
  })
}

fn format_content(content: &str, mentions: &[ChatMention]) -> String {
  let mut out = String::with_capacity(content.len());
  let mut rest = content;

  while let Some(ch) = rest.chars().next() {
    if let Some(after) = rest.strip_prefix("||")
      && let Some(end) = after.find("||")
    {
      out.push_str("[spoiler]");
      rest = &after[end + 2..];
      continue;
    }

    if ch == '<'
      && let Some(end) = rest.find('>')
      && let Some(text) = replace_token(&rest[1..end], mentions)
    {
      out.push_str(&text);
      rest = &rest[end + 1..];
      continue;
    }

    out.push(ch);
    rest = &rest[ch.len_utf8()..];
  }

  out
}

fn replace_token(inner: &str, mentions: &[ChatMention]) -> Option<String> {
  let is_id = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());

  if let Some(id) = inner.strip_prefix("@&") {
    return is_id(id).then(|| "@role".to_string());
  }
  if let Some(id) = inner.strip_prefix("@!").or_else(|| inner.strip_prefix('@')) {
    return is_id(id).then(|| format!("@{}", mention_name(id, mentions).unwrap_or("user")));
  }
  if let Some(id) = inner.strip_prefix('#') {
    return is_id(id).then(|| "#channel".to_string());
  }

  let emoji = inner
    .strip_prefix("a:")
    .or_else(|| inner.strip_prefix(':'))?;
  let (name, id) = emoji.split_once(':')?;
  (!name.is_empty() && is_id(id)).then(|| format!(":{name}:"))
}

fn mention_name<'a>(id: &str, mentions: &'a [ChatMention]) -> Option<&'a str> {
  let mention = mentions.iter().find(|m| m.id == id)?;
  [&mention.global_name, &mention.username]
    .into_iter()
    .flatten()
    .map(String::as_str)
    .find(|name| !name.is_empty())
}
