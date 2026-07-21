use freya::prelude::*;
use serde_json::json;

use orbolay_core::{
  app_state::AppState, payloads::SoundboardSoundPayload, util::bridge::BridgeMessage,
};

use crate::util::{
  scale::{GapsScaleExt, UiScale},
  theme::Theme,
};

fn guild_order(guild_id: &str, current: &str) -> u8 {
  if !current.is_empty() && guild_id == current {
    0
  } else if guild_id.is_empty() || guild_id == "0" {
    1
  } else {
    2
  }
}

#[derive(PartialEq)]
struct SoundButton {
  sound: SoundboardSoundPayload,
  app_state: State<AppState>,
  theme: Theme,
  ui_scale: f32,
}

impl Component for SoundButton {
  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    let mut app_state = self.app_state;
    let mut hovered = use_state(|| false);

    use_drop(move || {
      if *hovered.read() {
        Cursor::set(CursorIcon::default());
      }
    });

    let sound_id = self.sound.sound_id.clone();
    let source_guild_id = self.sound.guild_id.clone();
    let available = self.sound.available;
    let text = match &self.sound.emoji_name {
      Some(e) => e.to_string(),
      None => "?".into(),
    };
    let name = &self.sound.name;

    rect()
      .direction(Direction::Horizontal)
      .cross_align(Alignment::Center)
      .main_align(Alignment::Center)
      .width(Size::percent(33.3_f32))
      .height(Size::px(scale.px(40.0)))
      .margin(Gaps::new_all(2.).scaled(scale.factor()))
      .corner_radius(CornerRadius::new_all(self.theme.border_radius))
      .maybe(!available, |el| el.opacity(0.4_f32))
      .background(if *hovered.read() {
        self.theme.light_gray
      } else {
        self.theme.darkish_gray
      })
      .on_press(move |_| {
        if !available {
          return;
        }
        app_state.write().send(BridgeMessage {
          cmd: "PLAY_SOUNDBOARD_SOUND".to_string(),
          data: json!({
            "sound_id": sound_id,
            "source_guild_id": source_guild_id,
          }),
        });
      })
      .on_pointer_enter(move |_| {
        *hovered.write() = true;
        Cursor::set(CursorIcon::Pointer);
      })
      .on_pointer_leave(move |_| {
        *hovered.write() = false;
        Cursor::set(CursorIcon::default());
      })
      .overflow(Overflow::Clip)
      .child(
        rect()
          .direction(Direction::Horizontal)
          .main_align(Alignment::Center)
          .cross_align(Alignment::Center)
          .content(Content::wrap())
          .padding(Gaps::new_symmetric(4., 2.).scaled(scale.factor()))
          .child(
            rect()
              .padding(Gaps::new(0., 4., 0., 0.).scaled(scale.factor()))
              .child(
                label()
                  .color(self.theme.text_color)
                  .font_size(scale.px(14.))
                  .text(text),
              ),
          )
          .child(
            label()
              .font_size(scale.px(11.))
              .color(self.theme.text_color)
              .max_width(Size::fill())
              .max_lines(1)
              .text(name.clone())
              .text_overflow(TextOverflow::Ellipsis),
          ),
      )
  }
}

#[derive(PartialEq)]
struct GuildLabel {
  name: String,
  theme: Theme,
  ui_scale: f32,
}

impl Component for GuildLabel {
  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    label()
      .font_size(scale.px(11.))
      .width(Size::fill())
      .color(self.theme.text_color)
      .text(self.name.clone())
  }
}

#[derive(PartialEq)]
pub struct Soundboard {
  pub app_state: State<AppState>,
  pub theme: Theme,
  pub ui_scale: f32,
}

impl Component for Soundboard {
  fn render(&self) -> impl IntoElement {
    let scale = UiScale::new(self.ui_scale);
    let app_state = self.app_state;
    let (current_guild_id, cache, guild_names) = {
      let state = app_state.read();
      (
        state.current_guild_id.clone(),
        state.soundboard_cache.clone(),
        state.guild_names.clone(),
      )
    };

    let mut guilds: Vec<(String, Vec<SoundboardSoundPayload>)> = cache.into_iter().collect();
    guilds.sort_by(|(a, _), (b, _)| {
      guild_order(a, &current_guild_id)
        .cmp(&guild_order(b, &current_guild_id))
        .then(b.cmp(a))
    });

    if guilds.is_empty() {
      rect()
        .direction(Direction::Vertical)
        .background(self.theme.gray)
        .corner_radius(CornerRadius::new_all(self.theme.border_radius))
        .max_width(Size::px(scale.px(400.0)))
        .margin(Gaps::new(0., 0., 8., 0.).scaled(scale.factor()))
        .padding(Gaps::new_all(16.).scaled(scale.factor()))
        .main_align(Alignment::Center)
        .cross_align(Alignment::Center)
        .child(
          label()
            .font_size(scale.px(13.))
            .color(self.theme.text_color)
            .text("No sounds available"),
        )
    } else {
      rect()
        .direction(Direction::Vertical)
        .background(self.theme.gray)
        .corner_radius(CornerRadius::new_all(self.theme.border_radius))
        .max_width(Size::px(scale.px(400.0)))
        .height(Size::px(scale.px(220.0)))
        .margin(Gaps::new(0., 0., 8., 0.).scaled(scale.factor()))
        .child(
          ScrollView::new()
            .width(Size::fill())
            .height(Size::fill())
            .child(
              guilds.into_iter().fold(
                rect()
                  .direction(Direction::Vertical)
                  .width(Size::fill())
                  .padding(Gaps::new_all(16.).scaled(scale.factor())),
                |col, (guild_name, guild_sounds)| {
                  let label = if guild_name.is_empty() {
                    "Default".to_string()
                  } else {
                    guild_names.get(&guild_name).cloned().unwrap_or(guild_name)
                  };
                  col
                    .child(GuildLabel {
                      name: label,
                      theme: self.theme,
                      ui_scale: scale.factor(),
                    })
                    .child(
                      guild_sounds.into_iter().fold(
                        rect()
                          .direction(Direction::Horizontal)
                          .content(Content::wrap())
                          .width(Size::fill())
                          .padding(Gaps::new(2., 0., 6., 0.).scaled(scale.factor())),
                        |row, mut sound| {
                          if let Some(guild_id) = &sound.guild_id
                            && !app_state.read().premium_type.has_nitro()
                            && guild_id != &"0".to_string()
                            && guild_id != &app_state.read().current_guild_id
                          {
                            sound.available = false;
                          }

                          row.child(SoundButton {
                            sound,
                            app_state,
                            theme: self.theme,
                            ui_scale: scale.factor(),
                          })
                        },
                      ),
                    )
                },
              ),
            ),
        )
    }
  }
}
