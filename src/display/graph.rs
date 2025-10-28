use super::colors::get_ansi_color;
use crate::config::ColorConfig;
use chrono::{Datelike, NaiveDate};
use serde_json::Value;

pub struct ContributionGraph {
  weeks: Vec<Week>,
}

#[derive(Clone)]
pub struct Week {
  pub contribution_days: Vec<Day>,
}

#[derive(Clone)]
pub struct Day {
  pub contribution_count: u32,
  #[allow(dead_code)]
  pub date: String,
}

impl ContributionGraph {
  pub fn from_json(data: &Value) -> Self {
    let weeks = data
      .as_array()
      .map(|arr| {
        arr
          .iter()
          .map(|week| Week {
            contribution_days: week["contributionDays"]
              .as_array()
              .map(|days| {
                days
                  .iter()
                  .map(|day| Day {
                    contribution_count: day["contributionCount"].as_u64().unwrap_or(0) as u32,
                    date: day["date"].as_str().unwrap_or("").to_string(),
                  })
                  .collect()
              })
              .unwrap_or_default(),
          })
          .collect()
      })
      .unwrap_or_default();

    Self { weeks }
  }

  pub fn from_grid(grid: Vec<Vec<u8>>) -> Self {
    // grid is 7 rows x N columns
    if grid.is_empty() || grid[0].is_empty() {
      return Self { weeks: Vec::new() };
    }

    let num_columns = grid[0].len();
    let mut weeks = Vec::new();

    for col_idx in 0..num_columns {
      let mut week_days = Vec::new();
      for row_idx in 0..7 {
        let intensity = if row_idx < grid.len() && col_idx < grid[row_idx].len() {
          grid[row_idx][col_idx]
        } else {
          0
        };

        week_days.push(Day {
          contribution_count: intensity as u32,
          date: format!("2023-01-{:02}", col_idx + 1),
        });
      }

      weeks.push(Week {
        contribution_days: week_days,
      });
    }

    Self { weeks }
  }

  pub fn render(
    &self,
    width: Option<usize>,
    height: Option<usize>,
    custom_box: &str,
    colors: &ColorConfig,
    show_date: bool,
    spaced: bool,
  ) -> Vec<String> {
    let mut lines = Vec::new();

    // Use specified width or default to 52 weeks
    let num_weeks = width.unwrap_or(52);
    let recent_weeks = self.get_recent_weeks(num_weeks);

    if show_date {
      let month_line = self.build_month_line(&recent_weeks);
      lines.push(month_line);
    }

    // Use specified height or default to 7 days (full week)
    let num_days = height.unwrap_or(7).min(7);

    for day_idx in 0..num_days {
      let mut row = String::from("    ");
      for week in &recent_weeks {
        if let Some(day) = week.contribution_days.get(day_idx) {
          let block = if spaced {
            self.get_contribution_block_spaced(day.contribution_count, custom_box, colors)
          } else {
            self.get_contribution_block(day.contribution_count, colors)
          };
          row.push_str(&block);
        }
      }
      row.push_str("\x1b[0m");
      lines.push(row);
    }

    lines
  }

  fn get_contribution_block(&self, count: u32, colors: &ColorConfig) -> String {
    let color = match count {
      0 => &colors.level_0,
      1..=2 => &colors.level_1,
      3..=6 => &colors.level_2,
      7..=12 => &colors.level_3,
      _ => &colors.level_4,
    };

    // Not-spaced mode: use background color for filled square (2 spaces)
    let bg_color = get_ansi_color(color).unwrap_or_default();
    let bg_ansi = if !bg_color.is_empty() && bg_color.starts_with("\x1b[38;2;") {
      // Convert foreground (38) to background (48)
      bg_color.replace("\x1b[38;2;", "\x1b[48;2;")
    } else {
      bg_color
    };
    format!("{}  \x1b[0m", bg_ansi)
  }

  fn get_contribution_block_spaced(
    &self,
    count: u32,
    custom_box: &str,
    colors: &ColorConfig,
  ) -> String {
    let color = match count {
      0 => &colors.level_0,
      1..=2 => &colors.level_1,
      3..=6 => &colors.level_2,
      7..=12 => &colors.level_3,
      _ => &colors.level_4,
    };

    // Spaced mode: use custom box character with foreground color + space
    let ansi_color = get_ansi_color(color).unwrap_or_default();
    format!("{}{}\x1b[0m ", ansi_color, custom_box)
  }

  fn get_recent_weeks(&self, limit: usize) -> Vec<Week> {
    if self.weeks.len() <= limit {
      self.weeks.clone()
    } else {
      self.weeks[self.weeks.len() - limit..].to_vec()
    }
  }

  fn build_month_line(&self, weeks: &[Week]) -> String {
    if weeks.is_empty() {
      return String::new();
    }

    let months = [
      "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let mut month_line = String::new();

    for (idx, week) in weeks.iter().enumerate() {
      if week.contribution_days.is_empty() {
        continue;
      }

      let first_day = &week.contribution_days[0];
      if let Ok(date) = NaiveDate::parse_from_str(&first_day.date, "%Y-%m-%d") {
        let current_month = date.month() as usize;

        if idx == 0 {
          month_line.push_str(months[current_month - 1]);
        } else {
          if let Some(prev_week) = weeks.get(idx - 1) {
            if !prev_week.contribution_days.is_empty() {
              if let Ok(prev_date) =
                NaiveDate::parse_from_str(&prev_week.contribution_days[0].date, "%Y-%m-%d")
              {
                let prev_month = prev_date.month() as usize;
                if current_month != prev_month {
                  let target_width = (idx + 1) * 2;
                  let current_width = month_line.len();
                  let month_name = months[current_month - 1];
                  let needed_space = (target_width - current_width - month_name.len()).max(1);
                  month_line.push_str(&" ".repeat(needed_space));
                  month_line.push_str(month_name);
                }
              }
            }
          }
        }
      }
    }

    format!("    {}", month_line)
  }

  #[allow(dead_code)]
  pub fn calculate_total_contributions(&self) -> u32 {
    self
      .weeks
      .iter()
      .flat_map(|w| &w.contribution_days)
      .map(|d| d.contribution_count)
      .sum()
  }

  #[allow(dead_code)]
  pub fn calculate_streaks(&self) -> (u32, u32) {
    let mut all_contributions: Vec<u32> = self
      .weeks
      .iter()
      .flat_map(|w| &w.contribution_days)
      .map(|d| d.contribution_count)
      .collect();

    all_contributions.reverse();

    let mut current_streak = 0;
    for &count in &all_contributions {
      if count > 0 {
        current_streak += 1;
      } else {
        break;
      }
    }

    let mut max_streak = 0;
    let mut temp_streak = 0;
    for &count in &all_contributions {
      if count > 0 {
        temp_streak += 1;
        max_streak = max_streak.max(temp_streak);
      } else {
        temp_streak = 0;
      }
    }

    (current_streak, max_streak)
  }
}
