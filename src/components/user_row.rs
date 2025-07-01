use freya::prelude::*;
use skia_safe::Color;

use crate::{
  app_state::AppState,
  config::CornerAlignment,
  user::{User, UserVoiceState},
  util::image::circular_with_border,
};

import_svg!(Deafened, "../../assets/deafened.svg", {
  height: "16",
  width: "16",
  margin: "0 6 0 0",
});
import_svg!(Muted, "../../assets/muted.svg", {
  height: "16",
  width: "16",
  margin: "0 6 0 0",
});

#[derive(Props, Clone, PartialEq)]
pub struct UserRowProps {
  pub app_state: Signal<AppState, SyncStorage>,
  pub user: User,
}

#[component]
fn avatar_icon(user: User) -> Element {
  rsx! {
    rect {
      width: "auto",
      height: "100%",
      // 50% of the height
      corner_radius: "25",
      image {
        sampling: "trilinear",
        image_data: dynamic_bytes(avatar(&user)),
      }
    }
  }
}

#[component]
fn user_label(user: User) -> Element {
  rsx! {
    rect {
      content: "flex",
      direction: "horizontal",
      main_align: "center",
      cross_align: "center",

      height: "70%",
      background: "#1e1f23",
      corner_radius: "5",
      margin: "0 6 0 6",

      rect {
        padding: "4",

        label {
          font_size: "14",
          color: "white",
          "{user.name}"
        }
      }

      if user.voice_state == UserVoiceState::Muted {
        Muted {}
      } else if user.voice_state == UserVoiceState::Deafened {
        Deafened {}
      }
    }
  }
}

pub fn user_row(props: UserRowProps) -> Element {
  rsx! {
    rect {
      content: "flex",
      direction: "horizontal",
      main_align: "start",
      cross_align: "center",
      height: "50",
      margin: "6",

      opacity: if props.user.voice_state != UserVoiceState::Speaking &&
        (props.app_state.read().config.voice_semitransparent && !props.app_state.read().is_open) {
          "0.5"
        } else {
          "1.0"
        },

      // Change order based on right/left alignment
      if CornerAlignment::from_str(&props.app_state.read().config.user_alignment).left {
        avatar_icon {
          user: props.user.clone()
        }
        user_label {
          user: props.user.clone()
        }
      } else {
        user_label {
          user: props.user.clone()
        }
        avatar_icon {
          user: props.user.clone()
        }
      }
    }
  }
}

fn avatar(user: &User) -> Vec<u8> {
  let border_color = match user.voice_state {
    UserVoiceState::Speaking => Some(Color::from_rgb(67, 147, 120)),
    UserVoiceState::Deafened | UserVoiceState::Muted => Some(Color::from_rgb(218, 62, 68)),
    _ => None,
  };

  circular_with_border(user.fetch_avatar().unwrap_or_default(), border_color).unwrap_or_default()
}
