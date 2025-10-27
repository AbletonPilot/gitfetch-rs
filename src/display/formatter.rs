use super::graph::ContributionGraph;
use crate::config::Config;
use anyhow::Result;
use serde_json::Value;

enum Layout {
  Minimal,
  Compact,
  Full,
}

pub struct DisplayFormatter {
  config: Config,
  terminal_width: usize,
  terminal_height: usize,
}

impl DisplayFormatter {
  pub fn new(config: Config) -> Result<Self> {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));

    Ok(Self {
      config,
      terminal_width: cols as usize,
      terminal_height: rows as usize,
    })
  }

  pub fn display(&self, username: &str, user_data: &Value, stats: &Value) -> Result<()> {
    let layout = self.determine_layout(username, user_data, stats);

    match layout {
      Layout::Minimal => self.display_minimal(username, stats)?,
      Layout::Compact => self.display_compact(username, user_data, stats)?,
      Layout::Full => self.display_full(username, user_data, stats)?,
    }

    println!();
    Ok(())
  }

  fn determine_layout(&self, _username: &str, _user_data: &Value, _stats: &Value) -> Layout {
    // 터미널 크기에 따라 레이아웃 결정
    if self.terminal_width < 80 {
      Layout::Minimal
    } else if self.terminal_width < 120 {
      Layout::Compact
    } else {
      Layout::Full
    }
  }

  fn display_minimal(&self, username: &str, stats: &Value) -> Result<()> {
    println!();
    self.display_contribution_graph(username, stats)?;
    Ok(())
  }

  fn display_compact(&self, username: &str, user_data: &Value, stats: &Value) -> Result<()> {
    println!();

    let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
    let total_contribs = graph.calculate_total_contributions();

    // 헤더
    let name = user_data["name"].as_str().unwrap_or(username);
    let header = format!(
      "\x1b[38;2;118;215;161m{}\x1b[0m - \x1b[38;2;255;184;108m{}\x1b[0m \x1b[38;2;118;215;161mcontributions this year\x1b[0m",
      name, total_contribs
    );
    println!("{}", header);
    println!();

    // 기여도 그래프
    self.display_contribution_graph(username, stats)?;

    // 성취
    println!();
    self.display_achievements(&graph)?;

    Ok(())
  }

  fn display_full(&self, username: &str, user_data: &Value, stats: &Value) -> Result<()> {
    println!();

    let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
    let total_contribs = graph.calculate_total_contributions();

    // 왼쪽: 기여도 그래프
    let graph_lines = self.get_contribution_graph_lines(username, stats)?;

    // 오른쪽: 사용자 정보
    let info_lines = self.format_user_info(username, user_data, total_contribs);
    let language_lines = self.format_languages(stats);
    let achievement_lines = self.format_achievements(&graph);

    let mut right_lines = info_lines;
    if !language_lines.is_empty() {
      right_lines.push(String::new());
      right_lines.extend(language_lines);
    }
    if !achievement_lines.is_empty() {
      right_lines.push(String::new());
      right_lines.extend(achievement_lines);
    }

    // 사이드바이사이드 출력
    let max_left_width = graph_lines
      .iter()
      .map(|l| self.display_width(l))
      .max()
      .unwrap_or(0);

    let max_lines = graph_lines.len().max(right_lines.len());

    for i in 0..max_lines {
      let left = if i < graph_lines.len() {
        &graph_lines[i]
      } else {
        ""
      };
      let left_width = self.display_width(left);
      let padding = " ".repeat(max_left_width.saturating_sub(left_width));

      let right = if i < right_lines.len() {
        &right_lines[i]
      } else {
        ""
      };

      println!("{}{}  {}", left, padding, right);
    }

    Ok(())
  }

  fn get_contribution_graph_lines(&self, _username: &str, stats: &Value) -> Result<Vec<String>> {
    let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
    let custom_box = self.config.custom_box.as_deref().unwrap_or("■");
    let show_date = self.config.show_date;

    let lines = graph.render(52, custom_box, &self.config.colors, show_date);
    Ok(lines)
  }

  fn display_contribution_graph(&self, username: &str, stats: &Value) -> Result<()> {
    let lines = self.get_contribution_graph_lines(username, stats)?;
    for line in lines {
      println!("{}", line);
    }
    Ok(())
  }

  fn format_user_info(
    &self,
    _username: &str,
    user_data: &Value,
    total_contribs: u32,
  ) -> Vec<String> {
    let mut lines = Vec::new();

    let name = user_data["name"].as_str().unwrap_or("Unknown");
    let header = format!(
      "\x1b[38;2;118;215;161m{}\x1b[0m - \x1b[38;2;255;184;108m{}\x1b[0m \x1b[38;2;118;215;161mcontributions this year\x1b[0m",
      name, total_contribs
    );
    lines.push(header);

    let plain = format!("{} - {} contributions this year", name, total_contribs);
    lines.push(self.colorize(&"─".repeat(plain.len()), "muted"));

    if let Some(bio) = user_data["bio"].as_str() {
      if !bio.is_empty() {
        let trimmed = bio.replace('\n', " ");
        let truncated = if trimmed.len() > 80 {
          &trimmed[..80]
        } else {
          &trimmed
        };
        lines.push(format!("{} {}", self.label("Bio"), truncated));
      }
    }

    if let Some(company) = user_data["company"].as_str() {
      if !company.is_empty() {
        lines.push(format!("{} {}", self.label("Company"), company));
      }
    }

    if let Some(blog) = user_data["blog"].as_str() {
      if !blog.is_empty() {
        lines.push(format!("{} {}", self.label("Website"), blog));
      }
    }

    lines
  }

  fn format_languages(&self, stats: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    let languages = match stats["languages"].as_object() {
      Some(langs) if !langs.is_empty() => langs,
      _ => return lines,
    };

    lines.push(self.colorize("TOP LANGUAGES", "header"));
    lines.push(self.colorize(&"─".repeat(13), "muted"));

    let mut lang_vec: Vec<_> = languages.iter().collect();
    lang_vec.sort_by(|a, b| {
      let a_val = a.1.as_f64().unwrap_or(0.0);
      let b_val = b.1.as_f64().unwrap_or(0.0);
      b_val.partial_cmp(&a_val).unwrap()
    });

    for (lang, percentage) in lang_vec.iter().take(5) {
      let pct = percentage.as_f64().unwrap_or(0.0);
      let lang_name = if lang.to_lowercase() == "jupyter notebook" {
        "Jupyter"
      } else {
        lang
      };

      let bar = self.render_progress_bar(pct, 24);
      lines.push(format!("{} {} {:5.1}%", self.label(lang_name), bar, pct));
    }

    lines
  }

  fn format_achievements(&self, graph: &ContributionGraph) -> Vec<String> {
    let mut lines = Vec::new();

    let (current_streak, max_streak) = graph.calculate_streaks();
    let total_contribs = graph.calculate_total_contributions();

    lines.push(self.colorize("ACHIEVEMENTS", "header"));
    lines.push(self.colorize(&"─".repeat(12), "muted"));

    if current_streak > 0 {
      let streak_text = if current_streak == 1 {
        "day".to_string()
      } else {
        "days".to_string()
      };
      lines.push(format!(
        "{} Current Streak  {} {}",
        self.colorize("🔥", "red"),
        current_streak,
        streak_text
      ));
    }

    if max_streak > 0 {
      let streak_text = if max_streak == 1 {
        "day".to_string()
      } else {
        "days".to_string()
      };
      lines.push(format!(
        "{} Best Streak     {} {}",
        self.colorize("⭐", "yellow"),
        max_streak,
        streak_text
      ));
    }

    if total_contribs >= 10000 {
      lines.push(format!(
        "{} Contributions   10k+",
        self.colorize("💎", "magenta")
      ));
    } else if total_contribs >= 5000 {
      lines.push(format!(
        "{} Contributions   5k+",
        self.colorize("👑", "yellow")
      ));
    } else if total_contribs >= 1000 {
      lines.push(format!(
        "{} Contributions   1k+",
        self.colorize("🎖️", "cyan")
      ));
    } else if total_contribs >= 100 {
      lines.push(format!(
        "{} Contributions   100+",
        self.colorize("🏆", "yellow")
      ));
    }

    lines
  }

  fn display_achievements(&self, graph: &ContributionGraph) -> Result<()> {
    let lines = self.format_achievements(graph);
    for line in lines {
      println!("{}", line);
    }
    Ok(())
  }

  fn render_progress_bar(&self, percentage: f64, width: usize) -> String {
    let width = width.max(1);
    let capped = percentage.max(0.0).min(100.0);
    let filled = ((capped / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    let empty = width - filled;

    let filled_segment = "▰".repeat(filled);
    let empty_segment = "▱".repeat(empty);

    let colored_filled = self.colorize(&filled_segment, "green");

    format!("{}{}", colored_filled, empty_segment)
  }

  fn label(&self, text: &str) -> String {
    let label = format!("{}:", text);
    let padded = format!("{:<12}", label);
    self.colorize(&padded, "bold")
  }

  fn colorize(&self, text: &str, color: &str) -> String {
    let color_code = match color {
      "header" => "\x1b[38;2;118;215;161m",
      "orange" => "\x1b[38;2;255;184;108m",
      "green" => "\x1b[38;2;80;250;123m",
      "muted" => "\x1b[38;2;68;71;90m",
      "bold" => "\x1b[1m",
      "red" => "\x1b[91m",
      "yellow" => "\x1b[93m",
      "cyan" => "\x1b[96m",
      "magenta" => "\x1b[95m",
      _ => "\x1b[0m",
    };

    format!("{}{}\x1b[0m", color_code, text)
  }

  fn display_width(&self, text: &str) -> usize {
    // ANSI 코드를 제거한 실제 표시 너비
    let ansi_pattern = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let clean = ansi_pattern.replace_all(text, "");
    clean.chars().count()
  }
}
