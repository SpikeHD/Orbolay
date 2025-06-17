use freya::prelude::*;
use serde_json::Value;

use crate::{app_state::AppState, user::{User, UserVoiceState}, websocket::BridgeMessage};

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

#[derive(Props, Clone, PartialEq)]
pub struct VoiceControlsProps {
  pub user: User,
  pub app_state: Signal<AppState, SyncStorage>,
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
  pub icon: Element,
  pub onclick: Callback<MouseEvent>,
}

fn control_button(props: ButtonProps) -> Element {
  rsx! {
    rect {
      content: "flex",
      direction: "vertical",
      main_align: "center",
      cross_align: "center",
      height: "100%",
      width: "auto",
      margin: "6",
      padding: "6",
      corner_radius: "10",
      onclick: move |e| {
        props.onclick.call(e);
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

      background: "#1e1f23",
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
        onclick: move |_| {
          (*props.app_state.write()).send(BridgeMessage {
            cmd: "DISCONNECT".to_string(),
            data: Value::Null,
          })
        }
      }
    }
  }
}
