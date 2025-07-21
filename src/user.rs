use crate::{AVATAR_CACHE, log};

static DEFAULT_AVATAR: &[u8] = include_bytes!("../assets/discordgrey.png");

#[derive(Clone, Debug, PartialEq)]
pub enum UserVoiceState {
  Speaking,
  NotSpeaking,
  Muted,
  Deafened,
  Unknown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct User {
  pub name: String,
  pub id: String,
  pub avatar: String,
  pub voice_state: UserVoiceState,
  pub streaming: bool,
}

impl User {
  pub fn fetch_avatar(&self) -> Result<Vec<u8>, ureq::Error> {
    if let Some(avatar) = AVATAR_CACHE().get(&self.avatar) {
      log!("Cache hit for avatar {}", self.avatar);
      return Ok(avatar.clone());
    }

    if self.avatar.is_empty() {
      return Ok(DEFAULT_AVATAR.to_vec());
    }

    let uri = format!(
      "https://cdn.discordapp.com/avatars/{}/{}.png?size=80",
      self.id, self.avatar
    );
    log!("Fetching avatar from {}", uri);
    let img = ureq::get(&uri).call()?.body_mut().read_to_vec()?;

    (*AVATAR_CACHE.write()).insert(self.avatar.clone(), img.clone());

    Ok(img)
  }
}
