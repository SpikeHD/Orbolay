// Remove some stuff that would break formatting
pub fn strip(text: impl AsRef<str>) -> String {
  text.as_ref().replace("\n", " ")
}

const WRAP_HEADROOM: f32 = 0.95;

fn char_px(c: char) -> f32 {
  match c as u32 {
    0x30..=0x39 => 8.0,
    0x41..=0x5A => 9.0,
    0x00..=0x7F => 6.5,
    0x0300..=0x036F | 0x200B..=0x200F | 0xFE00..=0xFE0F => 0.0,
    0x2600..=0x27BF | 0x1F000..=0x1FAFF => 18.0,
    0x1100..=0x115F
    | 0x2E80..=0xA4CF
    | 0xAC00..=0xD7A3
    | 0xF900..=0xFAFF
    | 0xFE30..=0xFE4F
    | 0xFF00..=0xFF60
    | 0xFFE0..=0xFFE6
    | 0x20000..=0x3FFFD => 12.5,
    _ => 7.0,
  }
}

pub fn estimate_lines(text: &str, width: f32, max: u32) -> u32 {
  let px: f32 = strip(text).chars().map(char_px).sum();
  ((px / (width * WRAP_HEADROOM)).ceil() as u32).clamp(1, max.max(1))
}

pub fn truncate_to_lines(text: &str, width: f32, max_lines: u32) -> String {
  let text = strip(text);
  let room = width * max_lines.max(1) as f32 * WRAP_HEADROOM;
  if text.chars().map(char_px).sum::<f32>() <= room {
    return text;
  }

  let budget = room - char_px('\u{2026}');
  let mut used = 0.0;
  let mut out = String::new();
  for c in text.chars() {
    used += char_px(c);
    if used > budget {
      break;
    }
    out.push(c);
  }

  format!("{}\u{2026}", out.trim_end())
}

pub fn censor(text: impl AsRef<str>) -> String {
  format!("{}...", text.as_ref().chars().next().unwrap_or('?'))
}
