use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserData {
  pub login: String,
  pub name: Option<String>,
  pub bio: Option<String>,
  pub company: Option<String>,
  pub blog: Option<String>,
  pub location: Option<String>,
  pub email: Option<String>,
  pub public_repos: u32,
  pub followers: u32,
  pub following: u32,
  pub created_at: Option<String>,
}
