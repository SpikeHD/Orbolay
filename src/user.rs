use crate::{AVATAR_CACHE, log};

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
}

impl User {
  pub fn fetch_avatar(&self) -> Result<Vec<u8>, ureq::Error> {
    if AVATAR_CACHE().contains_key(&self.avatar) {
      log!("Cache hit for avatar {}", self.avatar);
      // We can unwrap here because we know the key exists
      return Ok(AVATAR_CACHE().get(&self.avatar).unwrap().clone());
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
