use freya::prelude::*;

use crate::user::{User, UserVoiceState};

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
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
  pub icon: Element,
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

      {props.icon}
    }
  }
}

pub fn voice_controls(props: VoiceControlsProps) -> Element {
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
      if props.user.voice_state == UserVoiceState::Muted {
        control_button {
          icon: rsx! { Muted {} }
        }
      } else {
        control_button {
          icon: rsx! { Mute {} }
        }
      }

      // Deafen button
      if props.user.voice_state == UserVoiceState::Deafened {
        control_button {
          icon: rsx! { Deafened {} }
        }
      } else {
        control_button {
          icon: rsx! { Deafen {} }
        }
      }

      // Disconnect button
      control_button {
        icon: rsx! { Disconnect {} }
      }
    }
  }
}
