use anyhow::Result;
use chrono::{DateTime, Datelike, Duration, NaiveDate, Utc};
use git2::Repository;
use serde_json::{Value, json};
use std::collections::HashMap;

pub fn get_repo_path() -> Result<String> {
  let repo = Repository::discover(".")?;
  let path = repo
    .path()
    .parent()
    .ok_or_else(|| anyhow::anyhow!("Could not get repo path"))?;
  Ok(path.to_string_lossy().to_string())
}

pub fn analyze_local_repo() -> Result<Value> {
  let repo = Repository::discover(".")?;

  // Get current user from git config
  let config = repo.config()?;
  let user_name = config
    .get_string("user.name")
    .unwrap_or_else(|_| "Unknown".to_string());
  let user_email = config
    .get_string("user.email")
    .unwrap_or_else(|_| "".to_string());

  // Collect commits from last 52 weeks
  let mut revwalk = repo.revwalk()?;
  revwalk.push_head()?;

  let now = Utc::now();
  let one_year_ago = now - Duration::weeks(52);

  let mut commits_by_date: HashMap<NaiveDate, u32> = HashMap::new();
  let mut total_commits = 0;

  for oid in revwalk {
    let oid = oid?;
    let commit = repo.find_commit(oid)?;

    let commit_time = commit.time();
    let timestamp = commit_time.seconds();
    let datetime =
      DateTime::from_timestamp(timestamp, 0).ok_or_else(|| anyhow::anyhow!("Invalid timestamp"))?;

    // Only count commits from last 52 weeks
    if datetime < one_year_ago {
      continue;
    }

    let date = datetime.date_naive();
    *commits_by_date.entry(date).or_insert(0) += 1;
    total_commits += 1;
  }

  // Generate weeks data (52 weeks, 7 days each)
  let mut weeks = Vec::new();
  let mut current_date = one_year_ago.date_naive();

  // Start from Sunday
  while current_date.weekday().num_days_from_sunday() != 0 {
    current_date = current_date
      .pred_opt()
      .ok_or_else(|| anyhow::anyhow!("Date calculation error"))?;
  }

  for _ in 0..52 {
    let mut week_days = Vec::new();

    for _ in 0..7 {
      let count = commits_by_date.get(&current_date).copied().unwrap_or(0);
      week_days.push(json!({
        "contributionCount": count,
        "date": current_date.format("%Y-%m-%d").to_string(),
      }));

      current_date = current_date
        .succ_opt()
        .ok_or_else(|| anyhow::anyhow!("Date calculation error"))?;
    }

    weeks.push(json!({
      "contributionDays": week_days
    }));
  }

  Ok(json!({
    "name": user_name,
    "email": user_email,
    "bio": format!("Local repository: {}", get_repo_path()?),
    "company": "",
    "website": "",
    "total_repos": 1,
    "total_stars": 0,
    "total_forks": 0,
    "languages": {},
    "contribution_graph": weeks,
    "current_streak": 0,
    "longest_streak": 0,
    "total_contributions": total_commits,
    "pull_requests": {
      "open": 0,
      "awaiting_review": 0,
      "mentions": 0
    },
    "issues": {
      "assigned": 0,
      "created": 0,
      "mentions": 0
    },
  }))
}
