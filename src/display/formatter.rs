use super::graph::ContributionGraph;
use crate::config::Config;
use anyhow::Result;
use serde_json::Value;

#[derive(Debug)]
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

    let available_height = self.terminal_height.saturating_sub(2).max(10);

    for layout in layouts {
      let (width, height) = self.calculate_layout_dimensions(username, user_data, stats, &layout);

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
    user_data: &Value,
    stats: &Value,
  ) -> (usize, usize) {
    let graph_width = self
      .visual_opts
      .width
      .unwrap_or_else(|| (self.terminal_width.saturating_sub(40).max(40) * 3) / 4);

    let graph_height = if !self.visual_opts.no_grid {
      // Graph height = days to show + 1 for month labels line
      self.visual_opts.height.unwrap_or(7) + 1
    } else {
      // Just header dimensions when no grid
      let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
      let total_contribs = graph.calculate_total_contributions();
      let name = user_data["name"].as_str().unwrap_or("unknown");
      let header_text = format!("{} - {} contributions this year", name, total_contribs);
      return (self.display_width(&header_text), 1);
    };

    // Calculate actual right side content
    let mut right_lines = Vec::new();

    if !self.visual_opts.no_account {
      let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
      let total_contribs = graph.calculate_total_contributions();
      let name = user_data["name"].as_str().unwrap_or("unknown");
      let info_text = format!("{} - {} contributions this year", name, total_contribs);
      right_lines.push(info_text);
    }

    if !self.visual_opts.no_achievements {
      let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
      let (current_streak, max_streak) = graph.calculate_streaks();
      let total_contribs = graph.calculate_total_contributions();

      if !right_lines.is_empty() {
        right_lines.push(String::new());
      }

      right_lines.push("ACHIEVEMENTS".to_string());
      right_lines.push("────────────".to_string());

      if current_streak > 0 {
        right_lines.push(format!("🔥 Current Streak  {} days", current_streak));
      }
      if max_streak > 0 {
        right_lines.push(format!("⭐ Best Streak     {} days", max_streak));
      }
      if total_contribs >= 100 {
        right_lines.push("🏆 Contributions   100+".to_string());
      }
    }

    let max_lines = graph_height.max(right_lines.len());
    let right_width = right_lines
      .iter()
      .map(|line| self.display_width(line))
      .max()
      .unwrap_or(0);

    // Add 2: one for display_compact's println!(), one for display()'s println!()
    (graph_width + 2 + right_width, max_lines + 2)
  }

  fn calculate_full_dimensions(
    &self,
    _username: &str,
    user_data: &Value,
    stats: &Value,
  ) -> (usize, usize) {
    let graph_width = self
      .visual_opts
      .width
      .unwrap_or_else(|| ((self.terminal_width.saturating_sub(10).max(50) * 3) / 4).max(50));

    // Calculate actual left side height by simulating rendering
    let left_height = if !self.visual_opts.no_grid {
      let base_height = self.visual_opts.height.unwrap_or(7);
      let mut total = base_height + 1; // +1 for month line

      // Actually calculate PR/Issues line counts
      if !self.visual_opts.no_pr || !self.visual_opts.no_issues {
        total += 1; // spacing line

        let pr_lines = if !self.visual_opts.no_pr {
          self.format_pull_requests(stats).len()
        } else {
          0
        };
        let issue_lines = if !self.visual_opts.no_issues {
          self.format_issues(stats).len()
        } else {
          0
        };

        // Sections displayed side-by-side, use max
        total += pr_lines.max(issue_lines);
      }
      total
    } else {
      0
    };

    // Calculate right side content with ACTUAL rendered lines
    let mut right_lines = Vec::new();

    if !self.visual_opts.no_account {
      let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
      let total_contribs = graph.calculate_total_contributions();

      // Use actual format_user_info to get real line count
      let user_info_lines = self.format_user_info("", user_data, stats, total_contribs);
      right_lines.extend(user_info_lines);
    }

    // Only show languages if terminal width >= 120
    if !self.visual_opts.no_languages && self.terminal_width >= 120 {
      let language_lines = self.format_languages(stats);
      if !language_lines.is_empty() {
        if !right_lines.is_empty() {
          right_lines.push(String::new());
        }
        right_lines.extend(language_lines);
      }
    }

    if !self.visual_opts.no_achievements {
      let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
      let achievement_lines = self.format_achievements(&graph);
      if !achievement_lines.is_empty() {
        if !right_lines.is_empty() {
          right_lines.push(String::new());
        }
        right_lines.extend(achievement_lines);
      }
    }

    let max_height = left_height.max(right_lines.len());

    // Calculate ACTUAL widths by rendering and measuring
    // We need to actually render the left side to get accurate widths
    let left_width = if !self.visual_opts.no_grid {
      // Render actual graph lines with the calculated width constraint
      let graph_lines = self
        .get_contribution_graph_lines_with_width(_username, stats, graph_width)
        .unwrap_or_default();

      // If PR/Issues are shown, we need to include those too
      let mut all_left_lines = graph_lines;

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

        if !pr_lines.is_empty() || !issue_lines.is_empty() {
          all_left_lines.push(String::new());

          let pr_width = pr_lines
            .iter()
            .map(|l| self.display_width(l))
            .max()
            .unwrap_or(0);
          let max_section_lines = pr_lines.len().max(issue_lines.len());

          for i in 0..max_section_lines {
            let pr_part = if i < pr_lines.len() { &pr_lines[i] } else { "" };
            let issue_part = if i < issue_lines.len() {
              &issue_lines[i]
            } else {
              ""
            };
            let pr_part_width = self.display_width(pr_part);
            let padding_len = pr_width.saturating_sub(pr_part_width) + 3;

            // Simulate the actual line format
            let line = format!("    {}{}{}", pr_part, " ".repeat(padding_len), issue_part);
            all_left_lines.push(line);
          }
        }
      }

      // Get max width of all left lines
      all_left_lines
        .iter()
        .map(|l| self.display_width(l))
        .max()
        .unwrap_or(0)
    } else {
      0
    };

    let right_width = right_lines
      .iter()
      .map(|line| self.display_width(line))
      .max()
      .unwrap_or(0);

    // Add 2: one for display_full's println!(), one for display()'s println!()
    (left_width + 2 + right_width, max_height + 2)
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
      self.get_contribution_graph_lines_with_width(username, stats, graph_width)?
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
        // Python adds empty line ONLY if right_side already has content
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

  fn combine_section_grid(&self, columns: &[Vec<String>], width_limit: usize) -> Vec<String> {
    let active_columns: Vec<&Vec<String>> = columns.iter().filter(|col| !col.is_empty()).collect();

    if active_columns.is_empty() {
      return vec![];
    }

    let indent = "    ";
    let gap = "   ";
    let gap_width = gap.len();
    let indent_width = indent.len();

    // Calculate column info: (column, max_width)
    let column_info: Vec<(&Vec<String>, usize)> = active_columns
      .iter()
      .map(|col| {
        let max_width = col
          .iter()
          .map(|line| self.display_width(line))
          .max()
          .unwrap_or(0);
        (*col, max_width)
      })
      .collect();

    // Split columns into rows based on width_limit
    let mut rows: Vec<Vec<(&Vec<String>, usize)>> = vec![];
    let mut current_row: Vec<(&Vec<String>, usize)> = vec![];
    let mut current_width = indent_width;

    for (col, width) in column_info {
      let projected = if current_row.is_empty() {
        width
      } else {
        width + gap_width
      };

      if !current_row.is_empty() && current_width + projected > width_limit {
        rows.push(current_row);
        current_row = vec![];
        current_width = indent_width;
      }

      if current_row.is_empty() {
        current_width += width;
      } else {
        current_width += gap_width + width;
      }

      current_row.push((col, width));
    }

    if !current_row.is_empty() {
      rows.push(current_row);
    }

    // Build combined lines
    let mut combined = vec![];
    for (row_idx, row) in rows.iter().enumerate() {
      let max_lines = row.iter().map(|(col, _)| col.len()).max().unwrap_or(0);

      for line_idx in 0..max_lines {
        let mut parts = vec![];
        for (col_idx, (col, width)) in row.iter().enumerate() {
          let text = if line_idx < col.len() {
            &col[line_idx]
          } else {
            ""
          };
          let text_width = self.display_width(text);
          let pad_width = width.saturating_sub(text_width);
          let pad = " ".repeat(pad_width);
          let spacer = if col_idx < row.len() - 1 { gap } else { "" };
          parts.push(format!("{}{}{}", text, pad, spacer));
        }

        combined.push(format!("{}{}", indent, parts.join("").trim_end()));
      }

      if row_idx < rows.len() - 1 {
        combined.push(String::new());
      }
    }

    combined
  }

  fn display_full(&self, username: &str, user_data: &Value, stats: &Value) -> Result<()> {
    println!();

    let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
    let total_contribs = graph.calculate_total_contributions();

    // Calculate graph width constraint (matching Python)
    let graph_width = self
      .visual_opts
      .width
      .unwrap_or_else(|| ((self.terminal_width.saturating_sub(10).max(50) * 3) / 4).max(50));

    // Left: contribution graph + PR/Issues below (only if --no-grid is not set)
    let mut graph_lines = if !self.visual_opts.no_grid {
      self.get_contribution_graph_lines_with_width(username, stats, graph_width)?
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

      // Check if PR and Issues can fit side-by-side within graph_width
      let section_columns: Vec<Vec<String>> = if !pr_lines.is_empty() && !issue_lines.is_empty() {
        let pr_width = pr_lines
          .iter()
          .map(|l| self.display_width(l))
          .max()
          .unwrap_or(0);
        let issue_width = issue_lines
          .iter()
          .map(|l| self.display_width(l))
          .max()
          .unwrap_or(0);
        let total_width = pr_width + issue_width + 3; // gap

        if total_width <= graph_width {
          vec![pr_lines, issue_lines]
        } else {
          // If both exist but don't fit side-by-side, don't show either (matching Python)
          vec![]
        }
      } else if !pr_lines.is_empty() {
        vec![pr_lines]
      } else if !issue_lines.is_empty() {
        vec![issue_lines]
      } else {
        vec![]
      };

      // Combine sections side-by-side
      if !section_columns.is_empty() {
        if !graph_lines.is_empty() {
          graph_lines.push(String::new()); // Add spacing
        }

        let combined = self.combine_section_grid(&section_columns, graph_width);
        graph_lines.extend(combined);
      }
    }

    // Right: user information
    let mut right_lines = vec![];

    if !self.visual_opts.no_account {
      right_lines.extend(self.format_user_info(username, user_data, stats, total_contribs));
    }

    // Only show languages if terminal width >= 120 (matching Python behavior)
    if !self.visual_opts.no_languages && self.terminal_width >= 120 {
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
    // Show month labels (date line at top of graph)
    let show_date = true;
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

  fn get_contribution_graph_lines_with_width(
    &self,
    _username: &str,
    stats: &Value,
    width_constraint: usize,
  ) -> Result<Vec<String>> {
    let graph = ContributionGraph::from_json(&stats["contribution_graph"]);
    let custom_box = self.config.custom_box.as_deref().unwrap_or("■");
    let show_date = true;
    let spaced = self.visual_opts.spaced;

    // Calculate max weeks that fit in width_constraint
    // Each week is 2 chars wide (■ ), plus 4 char margin
    let block_width = 2;
    let header_margin = 4;
    let available_for_graph = width_constraint.saturating_sub(header_margin);
    let max_weeks = (available_for_graph / block_width).max(13).min(52);

    // Use calculated weeks as width
    let lines = graph.render(
      Some(max_weeks),
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

    // Build achievement entries
    let mut entries = Vec::new();

    if current_streak > 0 {
      let streak_text = if current_streak == 1 {
        format!("{} day", current_streak)
      } else {
        format!("{} days", current_streak)
      };
      entries.push((
        format!("{} Current Streak", self.colorize("🔥", "red")),
        streak_text,
      ));
    }

    if max_streak > 0 {
      let streak_text = if max_streak == 1 {
        format!("{} day", max_streak)
      } else {
        format!("{} days", max_streak)
      };
      entries.push((
        format!("{} Best Streak", self.colorize("⭐", "yellow")),
        streak_text,
      ));
    }

    if total_contribs >= 10000 {
      entries.push((
        format!("{} Contributions", self.colorize("💎", "magenta")),
        "10k+".to_string(),
      ));
    } else if total_contribs >= 5000 {
      entries.push((
        format!("{} Contributions", self.colorize("👑", "yellow")),
        "5k+".to_string(),
      ));
    } else if total_contribs >= 1000 {
      entries.push((
        format!("{} Contributions", self.colorize("🎖️", "cyan")),
        "1k+".to_string(),
      ));
    } else if total_contribs >= 100 {
      entries.push((
        format!("{} Contributions", self.colorize("🏆", "yellow")),
        "100+".to_string(),
      ));
    }

    if !entries.is_empty() {
      let title = "ACHIEVEMENTS";
      lines.push(self.colorize(title, "header"));
      lines.push(self.colorize(&"─".repeat(title.len()), "muted"));

      // Calculate max label width (without ANSI codes)
      let label_width = entries
        .iter()
        .map(|(label, _)| self.display_width(label))
        .max()
        .unwrap_or(0);

      for (label, value) in entries {
        let label_len = self.display_width(&label);
        let padding = " ".repeat(label_width.saturating_sub(label_len));
        lines.push(format!("{}{}  {}", label, padding, value));
      }
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

    // Use unicode-width to properly calculate width for CJK characters
    use unicode_width::UnicodeWidthStr;
    clean.width()
  }

  fn truncate_text(&self, text: &str, max_width: usize) -> String {
    if self.display_width(text) <= max_width {
      return text.to_string();
    }

    let ellipsis = '…';
    let mut truncated = String::new();
    for ch in text.chars() {
      let test = format!("{}{}{}", truncated, ch, ellipsis);
      if self.display_width(&test) > max_width {
        break;
      }
      truncated.push(ch);
    }
    format!("{}{}", truncated, ellipsis)
  }

  fn format_pull_requests(&self, stats: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    let prs = match stats.get("pull_requests") {
      Some(pr_data) if pr_data.is_object() => pr_data,
      _ => return lines,
    };

    lines.push(self.colorize("PULL REQUESTS", "header"));
    lines.push(self.colorize(&"─".repeat(13), "muted"));

    // Calculate label width (matching Python: max label length + 2 for colon and space)
    let labels = ["Awaiting Review", "Your Open PRs", "Mentions"];
    let label_width = labels.iter().map(|s| s.len()).max().unwrap_or(0) + 2;

    for (label, key) in [
      ("Awaiting Review", "awaiting_review"),
      ("Your Open PRs", "open"),
      ("Mentions", "mentions"),
    ] {
      let data = prs.get(key);
      let total = data
        .and_then(|d| d.get("total_count"))
        .and_then(|t| t.as_i64())
        .unwrap_or(0);

      let label_text = format!("{}:", label);
      let padded_label = format!("{:<width$}", label_text, width = label_width);
      lines.push(format!(
        "{} {}",
        self.colorize(&padded_label, "header"),
        total
      ));

      // Display items (max 3)
      let items = data
        .and_then(|d| d.get("items"))
        .and_then(|i| i.as_array())
        .map(|arr| &arr[..arr.len().min(3)])
        .unwrap_or(&[]);

      if items.is_empty() {
        lines.push(format!("  {}", self.colorize("• None", "muted")));
      } else {
        for item in items {
          let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("");
          let repo = item.get("repo").and_then(|r| r.as_str()).unwrap_or("");

          let mut bullet = format!("• {}", self.truncate_text(title, 24));
          if !repo.is_empty() {
            bullet.push_str(&format!(" ({})", self.truncate_text(repo, 16)));
          }
          lines.push(format!("  {}", bullet));
        }
      }
    }

    lines
  }

  fn format_issues(&self, stats: &Value) -> Vec<String> {
    let mut lines = Vec::new();

    let issues = match stats.get("issues") {
      Some(issue_data) if issue_data.is_object() => issue_data,
      _ => return lines,
    };

    lines.push(self.colorize("ISSUES", "header"));
    lines.push(self.colorize(&"─".repeat(6), "muted"));

    // Calculate label width (matching Python: max label length + 2 for colon and space)
    let labels = ["Assigned", "Created (open)", "Mentions"];
    let label_width = labels.iter().map(|s| s.len()).max().unwrap_or(0) + 2;

    for (label, key) in [
      ("Assigned", "assigned"),
      ("Created (open)", "created"),
      ("Mentions", "mentions"),
    ] {
      let data = issues.get(key);
      let total = data
        .and_then(|d| d.get("total_count"))
        .and_then(|t| t.as_i64())
        .unwrap_or(0);

      let label_text = format!("{}:", label);
      let padded_label = format!("{:<width$}", label_text, width = label_width);
      lines.push(format!(
        "{} {}",
        self.colorize(&padded_label, "header"),
        total
      ));

      // Display items (max 3)
      let items = data
        .and_then(|d| d.get("items"))
        .and_then(|i| i.as_array())
        .map(|arr| &arr[..arr.len().min(3)])
        .unwrap_or(&[]);

      if items.is_empty() {
        lines.push(format!("  {}", self.colorize("• None", "muted")));
      } else {
        for item in items {
          let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("");
          let repo = item.get("repo").and_then(|r| r.as_str()).unwrap_or("");

          let mut bullet = format!("• {}", self.truncate_text(title, 24));
          if !repo.is_empty() {
            bullet.push_str(&format!(" ({})", self.truncate_text(repo, 16)));
          }
          lines.push(format!("  {}", bullet));
        }
      }
    }

    lines
  }
}
