use freya::prelude::*;
use serde_json::Value;

use crate::{
  app_state::AppState,
  user::{User, UserVoiceState},
  util::colors,
  websocket::BridgeMessage,
};

import_svg!(Deafened, "../../assets/deafened.svg", {
  height: "24",
  width: "24",
});
import_svg!(Deafen, "../../assets/deafen.svg", {
  height: "24",
  width: "24",
});
import_svg!(Muted, "../../assets/muted.svg", {
  height: "24",
  width: "24",
});
import_svg!(Mute, "../../assets/mute.svg", {
  height: "24",
  width: "24",
});
import_svg!(Disconnect, "../../assets/disconnect.svg", {
  height: "24",
  width: "24",
});
import_svg!(StopStream, "../../assets/stopstream.svg", {
  height: "24",
  width: "24",
});

#[derive(Props, Clone, PartialEq)]
pub struct VoiceControlsProps {
  pub user: User,
  pub app_state: Signal<AppState, SyncStorage>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
  pub icon: Element,
  pub is_red: bool,
  pub onclick: Callback<MouseEvent>,
}

fn control_button(props: ButtonProps) -> Element {
  let mut hovered = use_signal(|| false);

  rsx! {
    rect {
      content: "flex",
      direction: "vertical",
      main_align: "center",
      cross_align: "center",
      height: "100%",
      width: "20%",
      margin: "6",
      padding: "6",
      corner_radius: "10",
      background: if *hovered.read() {
        if props.is_red {
          colors::RED_GRAY
        } else {
          colors::LIGHT_GRAY
        }
      } else {
        "transparent"
      },

      onclick: move |e| {
        props.onclick.call(e);
      },
      onmouseenter: move |_| {
        (*hovered.write()) = true;
      },
      onmouseleave: move |_| {
        (*hovered.write()) = false;
      },

      {props.icon}
    }
  }
}

pub fn voice_controls(mut props: VoiceControlsProps) -> Element {
  rsx! {
    rect {
      content: "flex",
      direction: "horizontal",
      main_align: "center",
      cross_align: "center",
      height: "auto",
      max_height: "60",
      max_width: "400",

      background: colors::GRAY,
      corner_radius: "10",

      // Mute button
      control_button {
        icon: rsx! {
          if props.user.voice_state == UserVoiceState::Muted || props.user.voice_state == UserVoiceState::Deafened {
            Muted {}
          } else {
            Mute {}
          }
        },
        is_red: props.user.voice_state == UserVoiceState::Muted || props.user.voice_state == UserVoiceState::Deafened,
        onclick: move |_| {
          (*props.app_state.write()).send(BridgeMessage {
            cmd: "TOGGLE_MUTE".to_string(),
            data: Value::Null,
          })
        }
      }

      // Deafen button
      control_button {
        icon: rsx! {
          if props.user.voice_state == UserVoiceState::Deafened {
            Deafened {}
          } else {
            Deafen {}
          }
        },
        is_red: props.user.voice_state == UserVoiceState::Deafened,
        onclick: move |_| {
          (*props.app_state.write()).send(BridgeMessage {
            cmd: "TOGGLE_DEAF".to_string(),
            data: Value::Null,
          })
        }
      }

      // Disconnect button
      control_button {
        icon: rsx! { Disconnect {} },
        is_red: true,
        onclick: move |_| {
          (*props.app_state.write()).send(BridgeMessage {
            cmd: "DISCONNECT".to_string(),
            data: Value::Null,
          })
        }
      }

      // Stop stream button
      if props.user.streaming {
        control_button {
          icon: rsx! { StopStream {} },
          is_red: true,
          onclick: move |_| {
            (*props.app_state.write()).send(BridgeMessage {
              cmd: "STOP_STREAM".to_string(),
              data: Value::Null,
            })
          }
        }
      }
    }
  }
}
