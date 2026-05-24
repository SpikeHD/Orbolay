use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::{LazyLock, Mutex}};

use bytes::Bytes;
use freya::elements::image::{image as img_fn, Image as FreyaImage, ImageHolder};
use freya::engine::prelude::{
  ClipOp, DirectContext, EncodedImageFormat, Paint, PaintStyle, PathBuilder,
  SkColor, SkData, SkImage, SkPoint, SkRect,
  raster_n32_premul,
};

use crate::log;

static AVATAR_CACHE: LazyLock<Mutex<HashMap<String, Vec<u8>>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

pub static DEFAULT_AVATAR: &[u8] = include_bytes!("../../assets/discordgrey.png");

const OUTPUT_RES: (i32, i32) = (256, 256);

pub fn image_from_bytes(bytes: Vec<u8>) -> FreyaImage {
  let data = if bytes.is_empty() { DEFAULT_AVATAR } else { &bytes };
  let sk_image = SkImage::from_encoded(SkData::new_copy(data))
    .or_else(|| SkImage::from_encoded(SkData::new_copy(DEFAULT_AVATAR)))
    .expect("Failed to decode avatar and fallback image");
  img_fn(ImageHolder {
    image: Rc::new(RefCell::new(sk_image)),
    bytes: Bytes::from(bytes),
  })
}

pub fn circular_with_border(
  image: Vec<u8>,
  border: Option<SkColor>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  let original_image =
    SkImage::from_encoded(SkData::new_copy(&image)).ok_or("Failed to decode image")?;
  let (orig_width, orig_height) = (original_image.width(), original_image.height());
  let scale_factor = OUTPUT_RES.0 as f32 / orig_width as f32;

  let scaled_width = (orig_width as f32 * scale_factor) as i32;
  let scaled_height = (orig_height as f32 * scale_factor) as i32;

  let mut scaled_surface =
    raster_n32_premul((scaled_width, scaled_height)).ok_or("Failed to create scaled surface")?;
  let scaled_canvas = scaled_surface.canvas();

  let dst_rect = SkRect::from_xywh(0.0, 0.0, scaled_width as f32, scaled_height as f32);
  scaled_canvas.draw_image_rect(original_image, None, dst_rect, &Paint::default());

  let scaled_image = scaled_surface.image_snapshot();
  let mut surface =
    raster_n32_premul((scaled_width, scaled_height)).ok_or("Failed to create final surface")?;
  let canvas = surface.canvas();

  let radius = (scaled_width.min(scaled_height) as f32) / 2.0;
  let mut path_builder = PathBuilder::new();
  path_builder.add_circle(SkPoint::new(radius, radius), radius, None);
  let clip_path = path_builder.snapshot();

  canvas.clip_path(&clip_path, ClipOp::Intersect, None);
  canvas.draw_image(scaled_image, (0, 0), None);

  let mut border_paint = Paint::default();
  let border = border.unwrap_or_else(|| SkColor::from_argb(0, 0, 0, 0));

  border_paint.set_color(border);
  border_paint.set_anti_alias(true);
  border_paint.set_style(PaintStyle::Stroke);
  border_paint.set_stroke_width(32.);

  canvas.draw_path(&clip_path, &border_paint);

  Ok(
    surface
      .image_snapshot()
      .encode(None::<&mut DirectContext>, EncodedImageFormat::PNG, 100)
      .ok_or("Failed to encode image")?
      .as_bytes()
      .to_vec(),
  )
}

pub fn fetch_icon(url: &str, placeholder: bool) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  if let Some(avatar) = AVATAR_CACHE.lock().unwrap().get(url).cloned() {
    log!("Cache hit for image {}", url);
    return Ok(avatar);
  }

  if url.is_empty() && placeholder {
    return Ok(DEFAULT_AVATAR.to_vec());
  }

  log!("Fetching avatar from {}", url);
  let img = ureq::get(url).call()?.body_mut().read_to_vec()?;

  AVATAR_CACHE.lock().unwrap().insert(url.to_string(), img.clone());

  Ok(img)
}
