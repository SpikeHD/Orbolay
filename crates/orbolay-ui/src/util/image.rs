use std::{
  collections::HashMap,
  sync::{LazyLock, Mutex},
};

use bytes::Bytes;
use freya::elements::image::{Image as FreyaImage, ImageHandle, image as img_fn};
use freya::engine::prelude::{
  ClipOp, DirectContext, EncodedImageFormat, Paint, PaintStyle, PathBuilder, SkColor, SkData,
  SkImage, SkPoint, SkRect, raster_n32_premul,
};

use orbolay_logging::log;

static AVATAR_CACHE: LazyLock<Mutex<HashMap<String, Vec<u8>>>> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

struct CachedAvatar {
  image: SkImage,
  bytes: Bytes,
}

type AvatarImageCache = Mutex<HashMap<(String, Option<u32>), CachedAvatar>>;

static AVATAR_IMAGE_CACHE: LazyLock<AvatarImageCache> =
  LazyLock::new(|| Mutex::new(HashMap::new()));

pub static DEFAULT_AVATAR: &[u8] = include_bytes!("../../../../assets/discordgrey.png");

const OUTPUT_RES: (i32, i32) = (256, 256);

fn border_key(border: Option<SkColor>) -> Option<u32> {
  border.map(|c| {
    ((c.a() as u32) << 24) | ((c.r() as u32) << 16) | ((c.g() as u32) << 8) | (c.b() as u32)
  })
}

pub fn avatar_image(url: &str, border: Option<SkColor>) -> FreyaImage {
  let key = (url.to_string(), border_key(border));

  let hit = {
    let cache = AVATAR_IMAGE_CACHE.lock().unwrap();
    cache.get(&key).map(|c| (c.image.clone(), c.bytes.clone()))
  };

  if let Some((image, bytes)) = hit {
    return img_fn(ImageHandle { image, bytes });
  }

  let raw = fetch_icon(url, true).unwrap_or_default();
  let processed = circular_with_border(raw, border).unwrap_or_default();

  if processed.is_empty() {
    let bytes = Bytes::from_static(DEFAULT_AVATAR);
    let sk_image =
      SkImage::from_encoded(SkData::new_copy(&bytes)).expect("Failed to decode default avatar");
    return img_fn(ImageHandle {
      image: sk_image,
      bytes,
    });
  }

  let bytes = Bytes::from(processed);
  let sk_image =
    SkImage::from_encoded(SkData::new_copy(&bytes)).expect("Failed to decode processed avatar");

  AVATAR_IMAGE_CACHE.lock().unwrap().insert(
    key,
    CachedAvatar {
      image: sk_image.clone(),
      bytes: bytes.clone(),
    },
  );

  img_fn(ImageHandle {
    image: sk_image,
    bytes,
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

  AVATAR_CACHE
    .lock()
    .unwrap()
    .insert(url.to_string(), img.clone());

  Ok(img)
}
