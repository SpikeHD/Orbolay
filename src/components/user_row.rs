use freya::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct UserRowProps {
  pub speaking: bool,
  pub name: String,
  pub avatar: Vec<u8>,
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

      rect {
        width: "25%",
        height: "100%",
        // TODO put actual image here
        svg {
          height: "100%",
          width: "100%",
          svg_content: avatar_svg(props.speaking, props.avatar),
        }
      }

      rect {
        content: "flex",
        main_align: "center",
        cross_align: "center",

        height: "70%",
        background: "#1e1f23",
        corner_radius: "5",
        margin: "0 0 0 6",

        rect {
          padding: "4",

          label {
            font_size: "14",
            color: "white",
            "{props.name}"
          }
        }
      }

    }
  }
}

fn avatar_svg(speaking: bool, _image: Vec<u8>) -> String {
  let border_color = if speaking { "#439378" } else { "transparent" };

  format!(r#"
  <svg width="100" height="100" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
    <!-- Outer border circle -->
    <circle cx="50" cy="50" r="45" fill="transparent" stroke="{border_color}" stroke-width="4"/>
    
    <!-- Inner circle -->
    <circle cx="50" cy="50" r="40" fill="lightgray"/>
  </svg>
  "#)
}