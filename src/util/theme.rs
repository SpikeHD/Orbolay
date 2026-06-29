use freya::{engine::prelude::RGB, prelude::Color};

pub const RED_GRAY: Color = Color::new(0xFF3F2226);
pub const DARKISH_GRAY: Color = Color::new(0xFF242428);
pub const GRAY: Color = Color::new(0xFF1E1F23);
pub const LIGHT_GRAY: Color = Color::new(0xFF37373C);
pub const SUPERLIGHT_GRAY: Color = Color::new(0xFFB4B4B4);
pub const MUTED_GRAY: Color = Color::new(0xFF6B6B70);

pub const GREEN: Color = Color::new(0xFF01863B);
pub const TRANSPARENT: Color = Color::new(0x00000000);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
  pub gray: Color,
  pub darkish_gray: Color,
  pub light_gray: Color,
  pub superlight_gray: Color,
  pub muted_gray: Color,
  pub transparent_gray: Color,
  pub text_color: Color,
  pub border_radius: f32,
}

impl Theme {
  pub fn from_values(accent: Color, text_color: Color, border_radius: f32) -> Self {
    Self {
      gray: accent,
      darkish_gray: blend(DARKISH_GRAY, accent, 0.35),
      light_gray: blend(LIGHT_GRAY, accent, 0.45),
      superlight_gray: blend(SUPERLIGHT_GRAY, accent, 0.22),
      muted_gray: blend(MUTED_GRAY, accent, 0.22),
      transparent_gray: with_alpha(accent, 0x56),
      text_color,
      border_radius,
    }
  }
}

pub fn to_tuple(color: Color) -> (u8, u8, u8) {
  let RGB { r, g, b } = color.to_rgb();
  (r, g, b)
}

pub fn from_tuple(rgb: (u8, u8, u8)) -> Color {
  Color::from_rgb(rgb.0, rgb.1, rgb.2)
}

fn blend(base: Color, tint: Color, tint_strength: f32) -> Color {
  let RGB {
    r: base_r,
    g: base_g,
    b: base_b,
  } = base.to_rgb();
  let RGB {
    r: tint_r,
    g: tint_g,
    b: tint_b,
  } = tint.to_rgb();

  let mix = |base: u8, tint: u8| -> u8 {
    ((base as f32 * (1.0 - tint_strength)) + (tint as f32 * tint_strength)).round() as u8
  };

  Color::from_rgb(
    mix(base_r, tint_r),
    mix(base_g, tint_g),
    mix(base_b, tint_b),
  )
}

fn with_alpha(color: Color, alpha: u8) -> Color {
  let RGB { r, g, b } = color.to_rgb();
  Color::new(((alpha as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32)
}
