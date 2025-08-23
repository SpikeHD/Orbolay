use crate::payloads::UpdatePayload;

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
