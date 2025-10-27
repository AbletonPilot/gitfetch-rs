use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
  pub total_stars: u32,
  pub total_forks: u32,
  pub total_repos: u32,
  pub languages: HashMap<String, f64>,
  pub contribution_graph: serde_json::Value,
  pub current_streak: u32,
  pub longest_streak: u32,
  pub total_contributions: u32,
  pub pull_requests: PullRequestStats,
  pub issues: IssueStats,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PullRequestStats {
  pub awaiting_review: u32,
  pub open: u32,
  pub mentions: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IssueStats {
  pub assigned: u32,
  pub created: u32,
  pub mentions: u32,
}
