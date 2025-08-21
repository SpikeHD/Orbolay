use skia_safe::{
  ClipOp, Color, Data, EncodedImageFormat, Image, Paint, PaintStyle, Path, Point, Rect,
  surfaces::raster_n32_premul,
};

use crate::{log, AVATAR_CACHE};

static DEFAULT_AVATAR: &[u8] = include_bytes!("../../assets/discordgrey.png");

const OUTPUT_RES: (i32, i32) = (256, 256);

// Take in a square image, round it out with a border
pub fn circular_with_border(
  image: Vec<u8>,
  border: Option<Color>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let original_image =
    Image::from_encoded(Data::new_copy(&image)).ok_or("Failed to decode image")?;
  let (orig_width, orig_height) = (original_image.width(), original_image.height());
  let scale_factor = OUTPUT_RES.0 as f32 / orig_width as f32;

  // Create a new surface for the upscaled image
  let scaled_width = (orig_width as f32 * scale_factor) as i32;
  let scaled_height = (orig_height as f32 * scale_factor) as i32;

  let mut scaled_surface =
    raster_n32_premul((scaled_width, scaled_height)).ok_or("Failed to create scaled surface")?;
  let scaled_canvas = scaled_surface.canvas();

  // Draw the original image scaled up onto the new surface
  let dst_rect = Rect::from_xywh(0.0, 0.0, scaled_width as f32, scaled_height as f32);
  scaled_canvas.draw_image_rect(original_image, None, dst_rect, &Paint::default());

  let scaled_image = scaled_surface.image_snapshot();
  let mut surface =
    raster_n32_premul((scaled_width, scaled_height)).ok_or("Failed to create final surface")?;
  let canvas = surface.canvas();

  // Circular clipping path
  let mut clip_path = Path::new();
  let radius = (scaled_width.min(scaled_height) as f32) / 2.0;
  clip_path.add_circle(Point::new(radius, radius), radius, None);

  // Clip canvas, draw image
  canvas.clip_path(&clip_path, ClipOp::Intersect, None);
  canvas.draw_image(scaled_image.as_ref(), (0, 0), None);

  // Draw the border
  let mut border_paint = Paint::default();
  let border = if let Some(border) = border {
    border
  } else {
    Color::from_argb(0, 0, 0, 0)
  };

  border_paint.set_color(border);
  border_paint.set_anti_alias(true);
  border_paint.set_style(PaintStyle::Stroke);
  border_paint.set_stroke_width(32.);

  canvas.draw_path(&clip_path, &border_paint);

  Ok(
    surface
      .image_snapshot()
      .encode_to_data(EncodedImageFormat::PNG)
      .ok_or("Failed to encode image")?
      .as_bytes()
      .to_vec(),
  )
}

pub fn fetch_icon(
  url: &str,
  placeholder: bool,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  if let Some(avatar) = AVATAR_CACHE().get(url) {
    log!("Cache hit for image {}", url);
    return Ok(avatar.clone());
  }

  if url.is_empty() && placeholder {
    return Ok(DEFAULT_AVATAR.to_vec());
  }

  log!("Fetching avatar from {}", url);
  let img = ureq::get(url).call()?.body_mut().read_to_vec()?;

  (*AVATAR_CACHE.write()).insert(url.to_string(), img.clone());

  Ok(img)
}