use anyhow::Result;
use clap::Parser;

mod cache;
mod cli;
mod config;
mod display;
mod fetcher;
mod models;

use cache::CacheManager;
use cli::{Cli, interactive};
use config::ConfigManager;
use display::DisplayFormatter;

#[tokio::main]
async fn main() -> Result<()> {
  let args = Cli::parse();

  // Clear cache
  if args.clear_cache {
    let config_manager = ConfigManager::new()?;
    let cache = CacheManager::new(config_manager.config.cache_expiry_minutes)?;
    cache.clear()?;
    println!("Cache cleared successfully!");
    return Ok(());
  }

  // Config initialization
  let mut config_manager = ConfigManager::new()?;

  // Change provider
  if args.change_provider {
    println!("🔄 Changing git provider...\n");
    initialize_gitfetch(&mut config_manager).await?;
    println!("\n✅ Provider changed successfully!");
    return Ok(());
  }

  // Check initialization
  if !config_manager.is_initialized() {
    println!("🚀 Welcome to gitfetch! Let's set up your configuration.\n");
    initialize_gitfetch(&mut config_manager).await?;
    println!("\n✅ Configuration saved! You can now use gitfetch.\n");
  }

  // Apply CLI args to config
  if let Some(custom_box) = args.custom_box {
    config_manager.config.custom_box = Some(custom_box);
  }
  if args.no_date {
    config_manager.config.show_date = false;
  }

  // Create fetcher
  let provider = config_manager
    .get_provider()
    .ok_or_else(|| anyhow::anyhow!("Provider not configured"))?;
  let provider_url = config_manager
    .get_provider_url()
    .unwrap_or("https://api.github.com");
  let token = config_manager.get_token();

  let fetcher = fetcher::create_fetcher(provider, provider_url, token)?;

  // Determine username
  let username = if let Some(u) = args.username {
    u
  } else if let Some(u) = config_manager.get_default_username() {
    u.to_string()
  } else {
    fetcher.get_authenticated_user().await?
  };

  // Cache manager
  let cache_manager = CacheManager::new(config_manager.config.cache_expiry_minutes)?;

  // Fetch data
  let (user_data, stats) = if args.no_cache {
    let user_data = fetcher.fetch_user_data(&username).await?;
    let stats = fetcher
      .fetch_user_stats(&username, Some(&user_data))
      .await?;
    (user_data, stats)
  } else {
    match cache_manager.get_cached_user_data(&username)? {
      Some(cached_user) => {
        let cached_stats = cache_manager
          .get_cached_stats(&username)?
          .ok_or_else(|| anyhow::anyhow!("Cached stats not found"))?;
        (cached_user, cached_stats)
      }
      None => {
        let user_data = fetcher.fetch_user_data(&username).await?;
        let stats = fetcher
          .fetch_user_stats(&username, Some(&user_data))
          .await?;
        cache_manager.cache_user_data(&username, &user_data, &stats)?;
        (user_data, stats)
      }
    }
  };

  // Display
  let formatter = DisplayFormatter::new(config_manager.config)?;
  formatter.display(&username, &user_data, &stats)?;

  Ok(())
}

async fn initialize_gitfetch(config_manager: &mut ConfigManager) -> Result<()> {
  let provider = interactive::prompt_provider()?;
  config_manager.set_provider(provider.clone());

  match provider.as_str() {
    "github" => {
      config_manager.set_provider_url("https://api.github.com".to_string());
    }
    "gitlab" => {
      config_manager.set_provider_url("https://gitlab.com".to_string());
    }
    "gitea" => {
      let url = interactive::prompt_url(&provider)?;
      config_manager.set_provider_url(url);
      let token = interactive::prompt_token(&provider)?;
      if !token.is_empty() {
        config_manager.set_token(token);
      }
    }
    "sourcehut" => {
      config_manager.set_provider_url("https://git.sr.ht".to_string());
      let token = interactive::prompt_token(&provider)?;
      if !token.is_empty() {
        config_manager.set_token(token);
      }
    }
    _ => {}
  }

  let fetcher = fetcher::create_fetcher(
    &provider,
    config_manager.get_provider_url().unwrap_or(""),
    config_manager.get_token(),
  )?;

  let username = fetcher.get_authenticated_user().await?;
  println!("Using authenticated user: {}", username);
  config_manager.set_default_username(username);

  config_manager.config.cache_expiry_minutes = 15;

  config_manager.save()?;

  Ok(())
}
