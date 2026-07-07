use std::fmt::Display;

use freya::prelude::{Alignment, Gaps};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum AxisAlignment {
  Start,
  Center,
  End,
}

impl Display for AxisAlignment {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AxisAlignment::Start => write!(f, "start"),
      AxisAlignment::Center => write!(f, "center"),
      AxisAlignment::End => write!(f, "end"),
    }
  }
}

impl AxisAlignment {
  pub fn to_freya(&self) -> Alignment {
    match self {
      AxisAlignment::Start => Alignment::Start,
      AxisAlignment::Center => Alignment::Center,
      AxisAlignment::End => Alignment::End,
    }
  }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct CornerAlignment {
  pub x: AxisAlignment,
  pub y: AxisAlignment,
}

impl CornerAlignment {
  pub fn from_str(s: impl AsRef<str>) -> Self {
    match s.as_ref().to_ascii_lowercase().as_str() {
      "topleft" => CornerAlignment {
        x: AxisAlignment::Start,
        y: AxisAlignment::Start,
      },
      "topright" => CornerAlignment {
        x: AxisAlignment::End,
        y: AxisAlignment::Start,
      },
      "bottomleft" => CornerAlignment {
        x: AxisAlignment::Start,
        y: AxisAlignment::End,
      },
      "bottomright" => CornerAlignment {
        x: AxisAlignment::End,
        y: AxisAlignment::End,
      },
      "topcenter" => CornerAlignment {
        x: AxisAlignment::Center,
        y: AxisAlignment::Start,
      },
      "bottomcenter" => CornerAlignment {
        x: AxisAlignment::Center,
        y: AxisAlignment::End,
      },
      "centerleft" => CornerAlignment {
        x: AxisAlignment::Start,
        y: AxisAlignment::Center,
      },
      "centerright" => CornerAlignment {
        x: AxisAlignment::End,
        y: AxisAlignment::Center,
      },
      _ => CornerAlignment {
        x: AxisAlignment::Start,
        y: AxisAlignment::Start,
      },
    }
  }

  pub fn to_gaps(&self, offset_x: i32, offset_y: i32) -> Gaps {
    let (top, bottom) = match self.y {
      AxisAlignment::Start => (offset_y as f32, 0.),
      AxisAlignment::End => (0., offset_y as f32),
      AxisAlignment::Center => (0., 0.),
    };
    let (left, right) = match self.x {
      AxisAlignment::Start => (offset_x as f32, 0.),
      AxisAlignment::End => (0., offset_x as f32),
      AxisAlignment::Center => (0., 0.),
    };
    Gaps::new(top, right, bottom, left)
  }
}
