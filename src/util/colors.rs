use freya::{engine::prelude::RGB, prelude::Color};

pub const RED_GRAY: Color = Color::new(0xFF3F2226);
pub const DARKISH_GRAY: Color = Color::new(0xFF242428);
pub const GRAY: Color = Color::new(0xFF1E1F23);
pub const LIGHT_GRAY: Color = Color::new(0xFF37373C);
pub const SUPERLIGHT_GRAY: Color = Color::new(0xFFB4B4B4);
pub const MUTED_GRAY: Color = Color::new(0xFF6B6B70);
pub const TRANSPARENT_GRAY: Color = Color::new(0x56222222);
pub const GREEN: Color = Color::new(0xFF01863B);
pub const TRANSPARENT: Color = Color::new(0x00000000);

pub fn to_tuple(color: Color) -> (u8, u8, u8) {
  let RGB { r, g, b } = color.to_rgb();
  (r, g, b)
}

pub fn from_tuple(rgb: (u8, u8, u8)) -> Color {
  Color::from_rgb(rgb.0, rgb.1, rgb.2)
}
