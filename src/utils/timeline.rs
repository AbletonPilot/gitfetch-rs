use anyhow::Result;
use regex::Regex;
use std::process::Command;

pub fn get_git_timeline_graph(vertical: bool) -> Result<String> {
  let output = Command::new("git")
    .args(&[
      "--no-pager",
      "log",
      "--color=always",
      "--graph",
      "--all",
      "--pretty=format:\"\"",
    ])
    .output()?;

  if !output.status.success() {
    return Err(anyhow::anyhow!("Failed to execute git log"));
  }

  let mut text = String::from_utf8_lossy(&output.stdout)
    .to_string()
    .replace("\"", "");

  if vertical {
    return Ok(text);
  }

  // Horizontal mode: rotate the graph
  text = text
    .chars()
    .map(|c| match c {
      '\\' => '/',
      '/' => '\\',
      '|' => '-',
      _ => c,
    })
    .collect();

  let ansi_pattern = Regex::new(r"\x1b\[[0-9;]*m")?;

  let lines: Vec<_> = text.lines().collect();
  let mut parsed_lines = Vec::new();

  for line in lines {
    let parts: Vec<_> = ansi_pattern.split(line).collect();
    let codes: Vec<_> = ansi_pattern.find_iter(line).map(|m| m.as_str()).collect();

    let mut current_color = String::new();
    let mut parsed = Vec::new();

    for (i, seg) in parts.iter().enumerate() {
      for ch in seg.chars() {
        parsed.push((ch, current_color.clone()));
      }
      if i < codes.len() {
        let code = codes[i];
        current_color = if code == "\x1b[0m" {
          String::new()
        } else {
          code.to_string()
        };
      }
    }
    parsed_lines.push(parsed);
  }

  if parsed_lines.is_empty() {
    return Ok(String::new());
  }

  let max_len = parsed_lines
    .iter()
    .map(|line| line.len())
    .max()
    .unwrap_or(0);
  let padded: Vec<_> = parsed_lines
    .iter()
    .map(|line| {
      let mut padded = line.clone();
      padded.resize(max_len, (' ', String::new()));
      padded
    })
    .collect();

  let mut rotated = Vec::new();
  for col in (0..max_len).rev() {
    let mut new_row = Vec::new();
    for row in 0..padded.len() {
      new_row.push(padded[row][col].clone());
    }
    rotated.push(new_row);
  }

  let mut out_lines = Vec::new();
  for row in rotated {
    let mut cur_color = String::new();
    let mut out_line = String::new();

    for (ch, color) in row {
      if color != cur_color {
        out_line.push_str(&color);
        cur_color = color.clone();
      }
      out_line.push(ch);
    }

    if !cur_color.is_empty() {
      out_line.push_str("\x1b[0m");
    }
    out_lines.push(out_line);
  }

  Ok(out_lines.join("\n"))
}
