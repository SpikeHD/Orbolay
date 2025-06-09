use skia_safe::{
  ClipOp, Color, Data, EncodedImageFormat, Image, Paint, PaintStyle, Path, Point,
  surfaces::raster_n32_premul,
};

// Take in a square image, round it out with a border
pub fn circular_with_border(
  image: Vec<u8>,
  border: Option<Color>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let skia_image = Image::from_encoded(Data::new_copy(&image)).ok_or("Failed to decode image")?;
  let (width, height) = (skia_image.width(), skia_image.height());

  let mut surface = raster_n32_premul((width, height)).ok_or("Failed to create surface")?;
  let canvas = surface.canvas();

  // Circular clipping path
  let mut clip_path = Path::new();
  let radius = (width as f32) / 2.0;
  clip_path.add_circle(Point::new(radius, radius), radius, None);

  // Clip canvas, draw image
  canvas.clip_path(&clip_path, ClipOp::Intersect, None);
  canvas.draw_image(skia_image.as_ref(), (0, 0), None);

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
  border_paint.set_stroke_width(4.0);

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
