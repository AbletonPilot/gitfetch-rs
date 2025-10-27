use anyhow::Result;
use dialoguer::{Input, Select, theme::ColorfulTheme};

pub fn prompt_provider() -> Result<String> {
  let providers = vec!["GitHub", "GitLab", "Gitea/Forgejo/Codeberg", "Sourcehut"];

  let selection = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("Choose your git provider")
    .items(&providers)
    .default(0)
    .interact()?;

  let provider = match selection {
    0 => "github",
    1 => "gitlab",
    2 => "gitea",
    3 => "sourcehut",
    _ => unreachable!(),
  };

  Ok(provider.to_string())
}

pub fn prompt_url(provider: &str) -> Result<String> {
  Input::new()
    .with_prompt(format!("Enter {} instance URL", provider))
    .interact_text()
    .map_err(Into::into)
}

pub fn prompt_token(provider: &str) -> Result<String> {
  Input::new()
    .with_prompt(format!(
      "Enter {} API token (optional, press Enter to skip)",
      provider
    ))
    .allow_empty(true)
    .interact_text()
    .map_err(Into::into)
}
