use super::graph::ContributionGraph;
use crate::config::Config;
use anyhow::Result;
use serde_json::Value;

pub struct DisplayFormatter {
  config: Config,
}

impl DisplayFormatter {
  pub fn new(config: Config) -> Result<Self> {
    Ok(Self { config })
  }

  pub fn display(&self, username: &str, user_data: &Value, stats: &Value) -> Result<()> {
    println!("\n");

    self.display_header(username, user_data)?;
    println!();

    self.display_basic_stats(user_data, stats)?;
    println!();

    self.display_contribution_graph(stats)?;
    println!();

    Ok(())
  }

  fn display_header(&self, username: &str, user_data: &Value) -> Result<()> {
    let name = user_data["name"].as_str().unwrap_or(username);

    println!("\x1b[1;36m{}\x1b[0m", name);

    if let Some(bio) = user_data["bio"].as_str() {
      println!("{}", bio);
    }

    Ok(())
  }

  fn display_basic_stats(&self, user_data: &Value, stats: &Value) -> Result<()> {
    let total_repos = stats["total_repos"].as_u64().unwrap_or(0);
    let followers = user_data["followers"].as_u64().unwrap_or(0);
    let following = user_data["following"].as_u64().unwrap_or(0);

    let total_stars = stats["total_stars"].as_u64().unwrap_or(0);
    let total_forks = stats["total_forks"].as_u64().unwrap_or(0);
    let current_streak = stats["current_streak"].as_u64().unwrap_or(0);
    let longest_streak = stats["longest_streak"].as_u64().unwrap_or(0);
    let total_contributions = stats["total_contributions"].as_u64().unwrap_or(0);

    println!("\x1b[1;33mRepositories:\x1b[0m {}", total_repos);
    println!("\x1b[1;33mStars:\x1b[0m {}", total_stars);
    println!("\x1b[1;33mForks:\x1b[0m {}", total_forks);
    println!("\x1b[1;33mFollowers:\x1b[0m {}", followers);
    println!("\x1b[1;33mFollowing:\x1b[0m {}", following);
    println!(
      "\x1b[1;33mTotal Contributions:\x1b[0m {}",
      total_contributions
    );
    println!("\x1b[1;33mCurrent Streak:\x1b[0m {} days", current_streak);
    println!("\x1b[1;33mLongest Streak:\x1b[0m {} days", longest_streak);

    Ok(())
  }

  fn display_contribution_graph(&self, stats: &Value) -> Result<()> {
    let graph = ContributionGraph::from_json(&stats["contribution_graph"]);

    let custom_box = self.config.custom_box.as_deref().unwrap_or("■");
    let show_date = self.config.show_date;

    let lines = graph.render(52, custom_box, &self.config.colors, show_date);

    println!("\x1b[1;32mContribution Graph:\x1b[0m");
    for line in lines {
      println!("{}", line);
    }

    Ok(())
  }
}
