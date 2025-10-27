use super::colors::get_ansi_color;
use crate::config::ColorConfig;
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

  pub fn render(
    &self,
    _width: usize,
    custom_box: &str,
    colors: &ColorConfig,
    show_date: bool,
  ) -> Vec<String> {
    let mut lines = Vec::new();
    let recent_weeks = self.get_recent_weeks(52);

    if show_date {
      let month_line = self.build_month_line(&recent_weeks);
      lines.push(month_line);
    }

    for day_idx in 0..7 {
      let mut row = String::from("    ");
      for week in &recent_weeks {
        if let Some(day) = week.contribution_days.get(day_idx) {
          let block = self.get_contribution_block(day.contribution_count, custom_box, colors);
          row.push_str(&block);
        }
      }
      row.push_str("\x1b[0m");
      lines.push(row);
    }

    lines
  }

  fn get_contribution_block(&self, count: u32, custom_box: &str, colors: &ColorConfig) -> String {
    let color = match count {
      0 => &colors.level_0,
      1..=2 => &colors.level_1,
      3..=6 => &colors.level_2,
      7..=12 => &colors.level_3,
      _ => &colors.level_4,
    };

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

  fn build_month_line(&self, _weeks: &[Week]) -> String {
    String::from("    Jan  Feb  Mar  Apr  May  Jun  Jul  Aug  Sep  Oct  Nov  Dec")
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
