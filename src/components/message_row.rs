use freya::prelude::*;

use crate::{
  app_state::AppState,
  payloads::MessageNotification,
  util::{
    colors,
    image::{circular_with_border, fetch_icon},
    text::{strip, truncate},
  },
  websocket::BridgeMessage,
};

#[derive(Props, Clone, PartialEq)]
pub struct MessageRowProps {
  pub app_state: Signal<AppState, SyncStorage>,
  pub message: MessageNotification,
}

pub fn message_row(mut props: MessageRowProps) -> Element {
  rsx! {
    rect {
      content: "flex",
      direction: "horizontal",
      main_align: "start",
      cross_align: "center",
      height: "70",
      max_width: "400",
      margin: "6",
      corner_radius: "10",

      background: colors::GRAY,

      // Navigate when clicked
      onclick: move |_| {
        (*props.app_state.write()).send(BridgeMessage {
          cmd: "NAVIGATE".to_string(),
          data: serde_json::json!({
            "guild_id": props.message.guild_id,
            "channel_id": props.message.channel_id,
            "message_id": props.message.message_id,
          })
        })
      },

      rect {
        content: "flex",
        direction: "horizontal",
        main_align: "start",
        cross_align: "center",
        width: "auto",
        height: "100%",

        image {
          width: "54",
          height: "54",
          margin: "0 0 0 10",

          sampling: "trilinear",
          image_data: dynamic_bytes(icon(&props.message.icon.clone())),
        }

        rect {
          content: "flex",
          direction: "vertical",
          main_align: "start",
          cross_align: "start",
          height: "100%",
          width: "80%",
          margin: "6 0 6 6",

          label {
            font_size: "14",
            font_weight: "bold",
            color: "white",
            margin: "0 0 4 0",
            "{props.message.title}"
          }

          label {
            font_size: "14",
            color: colors::SUPERLIGHT_GRAY,
            "{truncate(strip(&props.message.body), 100)}"
          }
        }
      }
    }
  }
}

fn icon(url: &str) -> Vec<u8> {
  circular_with_border(fetch_icon(url, true).unwrap_or_default(), None).unwrap_or_default()
}
