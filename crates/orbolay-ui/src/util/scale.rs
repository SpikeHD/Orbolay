use freya::prelude::Gaps;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiScale(f32);

impl UiScale {
  pub fn new(scale: f32) -> Self {
    if scale.is_finite() && scale > 0.0 {
      Self(scale)
    } else {
      Self(1.0)
    }
  }

  pub fn factor(self) -> f32 {
    self.0
  }

  pub fn px(self, value: f32) -> f32 {
    value * self.0
  }

  pub fn int(self, value: i32) -> i32 {
    (value as f32 * self.0).round() as i32
  }
}

pub fn scale_f32(value: f32, scale: f32) -> f32 {
  UiScale::new(scale).px(value)
}

pub fn scale_i32(value: i32, scale: f32) -> i32 {
  UiScale::new(scale).int(value)
}

pub trait GapsScaleExt {
  fn scaled(self, scale: f32) -> Self;
}

impl GapsScaleExt for Gaps {
  fn scaled(self, scale: f32) -> Self {
    Gaps::new(
      self.top() * scale,
      self.right() * scale,
      self.bottom() * scale,
      self.left() * scale,
    )
  }
}
