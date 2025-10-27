use anyhow::Result;
use git2::Repository;

pub fn get_repo_path() -> Result<String> {
  let repo = Repository::discover(".")?;
  let path = repo
    .path()
    .parent()
    .ok_or_else(|| anyhow::anyhow!("Could not get repo path"))?;
  Ok(path.to_string_lossy().to_string())
}

pub fn get_current_branch() -> Result<String> {
  let repo = Repository::discover(".")?;
  let head = repo.head()?;
  let branch = head
    .shorthand()
    .ok_or_else(|| anyhow::anyhow!("Could not get current branch"))?;
  Ok(branch.to_string())
}
