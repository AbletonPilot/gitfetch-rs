use anyhow::Result;
use dialoguer::{Input, Select, theme::Theme};
use std::fmt;

/// Custom theme that matches Python gitfetch's provider selection UI
struct GitfetchTheme;

impl Theme for GitfetchTheme {
  fn format_prompt(&self, f: &mut dyn fmt::Write, prompt: &str) -> fmt::Result {
    write!(f, "{}", prompt)
  }

  fn format_select_prompt_item(
    &self,
    f: &mut dyn fmt::Write,
    text: &str,
    active: bool,
  ) -> fmt::Result {
    let indicator = if active { "●" } else { "○" };
    write!(f, "{} {}", indicator, text)
  }

  fn format_input_prompt(
    &self,
    f: &mut dyn fmt::Write,
    prompt: &str,
    default: Option<&str>,
  ) -> fmt::Result {
    match default {
      Some(default) => write!(f, "{} [{}]: ", prompt, default),
      None => write!(f, "{}: ", prompt),
    }
  }

  fn format_input_prompt_selection(
    &self,
    f: &mut dyn fmt::Write,
    prompt: &str,
    sel: &str,
  ) -> fmt::Result {
    write!(f, "{}: {}", prompt, sel)
  }
}

pub fn prompt_provider() -> Result<String> {
  let providers = vec!["GitHub", "GitLab", "Gitea/Forgejo/Codeberg", "Sourcehut"];

  println!("Choose your git provider:");
  println!();

  let selection = Select::with_theme(&GitfetchTheme)
    .items(&providers)
    .default(0)
    .interact()?;

  println!();
  println!("Use ↑/↓ arrows, ● = selected, Enter to confirm");

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
  Input::with_theme(&GitfetchTheme)
    .with_prompt(format!(
      "Enter your {} personal access token (optional, press Enter to skip)",
      provider
    ))
    .allow_empty(true)
    .interact_text()
    .map_err(Into::into)
}

pub fn prompt_cache_expiry() -> Result<usize> {
  let input: String = Input::with_theme(&GitfetchTheme)
    .with_prompt("Cache expiry in minutes (default: 15, Enter for default)")
    .allow_empty(true)
    .interact_text()?;

  if input.is_empty() {
    Ok(15)
  } else {
    match input.parse::<usize>() {
      Ok(minutes) if minutes >= 1 => Ok(minutes),
      _ => {
        println!("Cache expiry must be >= 1 min. Using default: 15");
        Ok(15)
      }
    }
  }
}
