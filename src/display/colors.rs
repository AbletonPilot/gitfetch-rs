use anyhow::Result;

pub fn hex_to_rgb(hex: &str) -> (u8, u8, u8) {
  let hex = hex.trim_start_matches('#');

  let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
  let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
  let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);

  (r, g, b)
}

pub fn get_ansi_color(hex: &str) -> Result<String> {
  let (r, g, b) = hex_to_rgb(hex);
  Ok(format!("\x1b[38;2;{};{};{}m", r, g, b))
}
