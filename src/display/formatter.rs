use super::graph::ContributionGraph;
use crate::config::Config;
use anyhow::Result;
use serde_json::Value;

enum Layout {
  Minimal,
  Compact,
  Full,
}

#[derive(Debug, Clone, Default)]
pub struct VisualOptions {
  pub graph_only: bool,
  pub spaced: bool,
  pub graph_timeline: bool,
  pub width: Option<usize>,
  pub height: Option<usize>,
  pub no_achievements: bool,
  pub no_languages: bool,
  pub no_issues: bool,
  pub no_pr: bool,
  pub no_account: bool,
  pub no_grid: bool,
}

pub struct DisplayFormatter {
  config: Config,
  terminal_width: usize,
  terminal_height: usize,
  visual_opts: VisualOptions,
}

impl DisplayFormatter {
  pub fn new(config: Config, visual_opts: VisualOptions) -> Result<Self> {
    let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));

    Ok(Self {
      config,
      terminal_width: cols as usize,
      terminal_height: rows as usize,
      visual_opts,
    })
  }

  pub fn display(&self, username: &str, user_data: &Value, stats: &Value) -> Result<()> {
    // Handle --graph-timeline option
    if self.visual_opts.graph_timeline {
      let timeline = crate::utils::timeline::get_git_timeline_graph(false)?;
      println!();
      println!("{}", timeline);
      return Ok(());
    }

    // Handle --graph-only option
    if self.visual_opts.graph_only {
      self.display_contribution_graph(username, stats)?;
      println!();
      return Ok(());
    }

    let layout = self.determine_layout(username, user_data, stats);

    match layout {
      Layout::Minimal => self.display_minimal(username, stats)?,
      Layout::Compact => self.display_compact(username, user_data, stats)?,
      Layout::Full => self.display_full(username, user_data, stats)?,
    }

    println!();
    Ok(())
  }

  fn determine_layout(&self, username: &str, user_data: &Value, stats: &Value) -> Layout {
    // Try layouts in order: full -> compact -> minimal
    // Choose the first one that fits in terminal dimensions
    let layouts = vec![Layout::Full, Layout::Compact, Layout::Minimal];

    for layout in layouts {
      let (width, height) = self.calculate_layout_dimensions(username, user_data, stats, &layout);
      let available_height = self.terminal_height.saturating_sub(2).max(10);

      if width <= self.terminal_width && height <= available_height {
        return layout;
      }
    }

    Layout::Minimal
  }

  fn calculate_layout_dimensions(
    &self,
    username: &str,
    user_data: &Value,
    stats: &Value,
    layout: &Layout,
  ) -> (usize, usize) {
    match layout {
      Layout::Minimal => self.calculate_minimal_dimensions(username, stats),
      Layout::Compact => self.calculate_compact_dimensions(username, user_data, stats),
      Layout::Full => self.calculate_full_dimensions(username, user_data, stats),
    }
  }

  fn calculate_minimal_dimensions(&self, _username: &str, _stats: &Value) -> (usize, usize) {
    if !self.visual_opts.no_grid {
      let width = self
        .visual_opts
        .width
        .unwrap_or(self.terminal_width.saturating_sub(4));
      let height = self.visual_opts.height.unwrap_or(7);
      (width, height)
    } else {
      // Just header
      (50, 2)
    }
  }

  fn calculate_compact_dimensions(
    &self,
    _username: &str,
    _user_data: &Value,
    _stats: &Value,
  ) -> (usize, usize) {
    let graph_width = self
      .visual_opts
      .width
      .unwrap_or_else(|| (self.terminal_width.saturating_sub(40).max(40) * 3) / 4);

    let graph_height = if !self.visual_opts.no_grid {
      self.visual_opts.height.unwrap_or(7)
    } else {
      2
    };

    let mut right_lines = 0;
    if !self.visual_opts.no_account {
      right_lines += 1; // User info header
    }
    if !self.visual_opts.no_achievements {
      right_lines += 5; // Achievements section
    }

    let max_lines = graph_height.max(right_lines);
    let right_width = 40; // Estimated right side width

    (graph_width + 2 + right_width, max_lines)
  }

  fn calculate_full_dimensions(
    &self,
    _username: &str,
    _user_data: &Value,
    stats: &Value,
  ) -> (usize, usize) {
    let graph_width = self
      .visual_opts
      .width
      .unwrap_or_else(|| ((self.terminal_width.saturating_sub(10).max(50) * 3) / 4).max(50));

    let graph_height = if !self.visual_opts.no_grid {
      let base_height = self.visual_opts.height.unwrap_or(7);
      let mut total = base_height + 1; // +1 for month line

      // Add PR/Issues sections if enabled
      if !self.visual_opts.no_pr || !self.visual_opts.no_issues {
        total += 6; // Estimated PR/Issues section height
      }
      total
    } else {
      2
    };

    let mut right_height = 0;
    if !self.visual_opts.no_account {
      right_height += 6; // User info
    }
    if !self.visual_opts.no_languages && self.terminal_width >= 120 {
      if let Some(langs) = stats["languages"].as_object() {
        right_height += 2 + langs.len().min(5); // Languages section
      }
    }
    if !self.visual_opts.no_achievements {
      right_height += 5; // Achievements
    }

    let max_height = graph_height.max(right_height);
    let right_width = 45; // Estimated right side width

    (graph_width + 2 + right_width, max_height)
  }

  fn display_minimal(&self, username: &str, stats: &Value) -> Result<()> {
    println!();
    self.display_contribution_graph(username, stats)?;
    Ok(())
  }

  fn display_compact(&self, username: &str, user_data: &Value, stats: &Value) -> Result<()> {
    println!();

    let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
    let graph_width = (self.terminal_width.saturating_sub(40).max(40) * 3) / 4;

    // Left side: graph lines
    let graph_lines = if !self.visual_opts.no_grid {
      self.get_contribution_graph_lines(username, stats)?
    } else {
      let total_contribs = graph.calculate_total_contributions();
      let name = user_data["name"].as_str().unwrap_or(username);
      vec![format!(
        "\x1b[38;2;118;215;161m{}\x1b[0m - \x1b[38;2;255;184;108m{}\x1b[0m \x1b[38;2;118;215;161mcontributions this year\x1b[0m",
        name, total_contribs
      )]
    };

    // Right side: compact user info + achievements (NO languages, NO PR/Issues)
    let mut right_lines = Vec::new();

    if !self.visual_opts.no_account {
      let total_contribs = graph.calculate_total_contributions();
      let name = user_data["name"].as_str().unwrap_or(username);
      right_lines.push(format!(
        "\x1b[38;2;118;215;161m{}\x1b[0m - \x1b[38;2;255;184;108m{}\x1b[0m \x1b[38;2;118;215;161mcontributions this year\x1b[0m",
        name, total_contribs
      ));
    }

    if !self.visual_opts.no_achievements {
      let achievement_lines = self.format_achievements(&graph);
      if !achievement_lines.is_empty() {
        if !right_lines.is_empty() {
          right_lines.push(String::new());
        }
        right_lines.extend(achievement_lines);
      }
    }

    // Display side-by-side
    let max_lines = graph_lines.len().max(right_lines.len());
    for i in 0..max_lines {
      let graph_part = if i < graph_lines.len() {
        &graph_lines[i]
      } else {
        ""
      };
      let graph_len = self.display_width(graph_part);
      let padding = " ".repeat(graph_width.saturating_sub(graph_len));

      let info_part = if i < right_lines.len() {
        &right_lines[i]
      } else {
        ""
      };

      println!("{}{}  {}", graph_part, padding, info_part);
    }

    Ok(())
  }

  fn display_full(&self, username: &str, user_data: &Value, stats: &Value) -> Result<()> {
    println!();

    let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
    let total_contribs = graph.calculate_total_contributions();

    // Left: contribution graph + PR/Issues below (only if --no-grid is not set)
    let mut graph_lines = if !self.visual_opts.no_grid {
      self.get_contribution_graph_lines(username, stats)?
    } else {
      vec![]
    };

    // Add PR/Issues sections to left side (below graph) if enabled
    if !self.visual_opts.no_pr || !self.visual_opts.no_issues {
      let pr_lines = if !self.visual_opts.no_pr {
        self.format_pull_requests(stats)
      } else {
        vec![]
      };

      let issue_lines = if !self.visual_opts.no_issues {
        self.format_issues(stats)
      } else {
        vec![]
      };

      // Combine PR and Issues side-by-side below graph
      if !pr_lines.is_empty() || !issue_lines.is_empty() {
        if !graph_lines.is_empty() {
          graph_lines.push(String::new()); // Add spacing
        }

        let pr_width = pr_lines
          .iter()
          .map(|l| self.display_width(l))
          .max()
          .unwrap_or(0);
        let max_section_lines = pr_lines.len().max(issue_lines.len());

        for i in 0..max_section_lines {
          let pr_part = if i < pr_lines.len() { &pr_lines[i] } else { "" };
          let pr_part_width = self.display_width(pr_part);
          let padding = " ".repeat(pr_width.saturating_sub(pr_part_width) + 3);

          let issue_part = if i < issue_lines.len() {
            &issue_lines[i]
          } else {
            ""
          };

          // Add 4-space indentation to match graph
          graph_lines.push(format!("    {}{}{}", pr_part, padding, issue_part));
        }
      }
    }

    // Right: user information
    let mut right_lines = vec![];

    if !self.visual_opts.no_account {
      right_lines.extend(self.format_user_info(username, user_data, stats, total_contribs));
    }

    if !self.visual_opts.no_languages {
      let language_lines = self.format_languages(stats);
      if !language_lines.is_empty() {
        if !right_lines.is_empty() {
          right_lines.push(String::new());
        }
        right_lines.extend(language_lines);
      }
    }

    if !self.visual_opts.no_achievements {
      let achievement_lines = self.format_achievements(&graph);
      if !achievement_lines.is_empty() {
        if !right_lines.is_empty() {
          right_lines.push(String::new());
        }
        right_lines.extend(achievement_lines);
      }
    }

    // Side-by-side output
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
    let show_date = true; // Always show month labels
    let spaced = self.visual_opts.spaced;

    // Use width/height options if specified, otherwise None (defaults: width=52, height=7)
    let lines = graph.render(
      self.visual_opts.width,
      self.visual_opts.height,
      custom_box,
      &self.config.colors,
      show_date,
      spaced,
    );

    Ok(lines)
  }

  fn display_contribution_graph(&self, username: &str, stats: &Value) -> Result<()> {
    let lines = self.get_contribution_graph_lines(username, stats)?;
    for line in lines {
      println!("{}", line);
    }
    Ok(())
  }

  pub fn display_simulation_from_grid(&self, grid: Vec<Vec<u8>>) -> Result<()> {
    let graph = ContributionGraph::from_grid(grid);
    let custom_box = self.config.custom_box.as_deref().unwrap_or("■");
    let show_date = false; // No date labels for simulations
    let spaced = self.visual_opts.spaced;

    let lines = graph.render(
      None, // Use full width
      None, // Use full height (7 days)
      custom_box,
      &self.config.colors,
      show_date,
      spaced,
    );

    for line in lines {
      println!("{}", line);
    }

    Ok(())
  }

  fn format_user_info(
    &self,
    _username: &str,
    user_data: &Value,
    stats: &Value,
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

    // Add stars amount
    if let Some(total_stars) = stats["total_stars"].as_i64() {
      lines.push(format!("{} {} ⭐", self.label("Stars"), total_stars));
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
    // Calculate actual display width after removing ANSI codes
    let ansi_pattern = regex::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let clean = ansi_pattern.replace_all(text, "");
    clean.chars().count()
  }

  fn format_pull_requests(&self, stats: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    let prs = match stats.get("pull_requests") {
      Some(pr_data) if pr_data.is_object() => pr_data,
      _ => return lines,
    };

    let open = prs["open"].as_i64().unwrap_or(0);
    let awaiting = prs["awaiting_review"].as_i64().unwrap_or(0);
    let mentions = prs["mentions"].as_i64().unwrap_or(0);

    lines.push(self.colorize("PULL REQUESTS", "header"));
    lines.push(self.colorize(&"─".repeat(13), "muted")); // Add underline

    lines.push(format!(
      "{} {}",
      self.colorize("Awaiting Review:", "header"),
      awaiting
    ));
    lines.push(format!("  {}", self.colorize("• None", "muted")));

    lines.push(format!(
      "{} {}",
      self.colorize("Your Open PRs:", "header"),
      open
    ));
    lines.push(format!("  {}", self.colorize("• None", "muted")));

    lines.push(format!(
      "{} {}",
      self.colorize("Mentions:", "header"),
      mentions
    ));
    lines.push(format!("  {}", self.colorize("• None", "muted")));

    lines
  }

  fn format_issues(&self, stats: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    let issues = match stats.get("issues") {
      Some(issue_data) if issue_data.is_object() => issue_data,
      _ => return lines,
    };

    let assigned = issues["assigned"].as_i64().unwrap_or(0);
    let created = issues["created"].as_i64().unwrap_or(0);
    let mentions = issues["mentions"].as_i64().unwrap_or(0);

    lines.push(self.colorize("ISSUES", "header"));
    lines.push(self.colorize(&"─".repeat(6), "muted")); // Add underline

    lines.push(format!(
      "{} {}",
      self.colorize("Assigned:", "header"),
      assigned
    ));
    lines.push(format!("  {}", self.colorize("• None", "muted")));

    lines.push(format!(
      "{} {}",
      self.colorize("Created (open):", "header"),
      created
    ));
    lines.push(format!("  {}", self.colorize("• None", "muted")));

    lines.push(format!(
      "{} {}",
      self.colorize("Mentions:", "header"),
      mentions
    ));
    lines.push(format!("  {}", self.colorize("• None", "muted")));

    lines
  }
}
