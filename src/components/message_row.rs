use freya::prelude::*;

use crate::{
  payloads::MessageNotification,
  util::{image::circular_with_border, strip, truncate},
};

#[derive(Props, Clone, PartialEq)]
pub struct MessageRowProps {
  pub message: MessageNotification,
}

pub fn message_row(props: MessageRowProps) -> Element {
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

      background: "#1e1f23",

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
          image_data: dynamic_bytes(icon(&props.message)),
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
            color: "#b4b4b4",
            "{truncate(strip(&props.message.body), 100)}"
          }
        }
      }
    }
  }
}

fn icon(message: &MessageNotification) -> Vec<u8> {
  circular_with_border(message.fetch_icon().unwrap_or_default(), None).unwrap_or_default()
}
