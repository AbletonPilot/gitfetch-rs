use anyhow::Result;
use clap::Parser;

mod cache;
mod cli;
mod config;
mod display;
mod fetcher;
mod models;
mod utils;

use cache::CacheManager;
use cli::{interactive, Cli};
use config::ConfigManager;
use display::DisplayFormatter;

async fn check_for_updates() -> Result<Option<String>> {
  let client = reqwest::Client::new();
  let response = client
    .get("https://crates.io/api/v1/crates/gitfetch-rs")
    .header("User-Agent", "gitfetch-rs")
    .timeout(std::time::Duration::from_secs(3))
    .send()
    .await?;

  if response.status().is_success() {
    let json: serde_json::Value = response.json().await?;
    if let Some(version) = json["crate"]["max_stable_version"].as_str() {
      Ok(Some(version.to_string()))
    } else {
      Ok(None)
    }
  } else {
    Ok(None)
  }
}

#[tokio::main]
async fn main() -> Result<()> {
  let args = Cli::parse();

  // Show version
  if args.version {
    println!("gitfetch-rs version: {}", env!("CARGO_PKG_VERSION"));

    // Check for updates from crates.io
    match check_for_updates().await {
      Ok(Some(latest)) => {
        let current = env!("CARGO_PKG_VERSION");
        if latest != current {
          println!("\x1b[93mUpdate available: {}", latest);
          println!("Get it at: https://crates.io/crates/gitfetch-rs");
          println!("Or run: cargo install cargo-update");
          println!("\tcargo install-update gitfetch-rs\x1b[0m");
        } else {
          println!("You are using the latest version.");
        }
      }
      Ok(None) => println!("Could not check for updates."),
      Err(_) => println!("Could not check for updates."),
    }

    return Ok(());
  }

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
    return Ok(());
  }

  // Apply CLI args to config
  if let Some(custom_box) = args.custom_box {
    config_manager.config.custom_box = Some(custom_box);
  }
  if args.no_date {
    config_manager.config.show_date = false;
  }

  // Clone config for later use (before any borrowing)
  let config_clone = config_manager.config.clone();

  // Visual options for display
  let visual_opts = display::VisualOptions {
    graph_only: args.graph_only,
    spaced: if args.not_spaced {
      false
    } else if args.spaced {
      true
    } else {
      true // Default to spaced mode
    },
    graph_timeline: args.graph_timeline,
    width: args.width,
    height: args.height,
    no_achievements: args.no_achievements,
    no_languages: args.no_languages,
    no_issues: args.no_issues,
    no_pr: args.no_pr,
    no_account: args.no_account,
    no_grid: args.no_grid,
  };

  // Handle text/shape simulation
  if args.text.is_some() || args.shape.is_some() {
    let formatter = DisplayFormatter::new(config_clone.clone(), visual_opts)?;

    if let Some(text) = args.text {
      use display::text_patterns::text_to_grid;
      let grid = text_to_grid(&text).map_err(|e| anyhow::anyhow!("{}", e))?;
      formatter.display_simulation_from_grid(grid)?;
    } else if let Some(shapes) = args.shape {
      use display::text_patterns::shape_to_grid;
      let grid = shape_to_grid(&shapes).map_err(|e| anyhow::anyhow!("{}", e))?;
      formatter.display_simulation_from_grid(grid)?;
    }

    return Ok(());
  }

  // Handle local mode
  if args.local {
    use std::path::Path;

    if !Path::new(".git").exists() {
      return Err(anyhow::anyhow!("Error: --local requires .git folder"));
    }

    let local_data = utils::git::analyze_local_repo()?;
    let username = local_data["name"].as_str().unwrap_or("Local User");

    let formatter = DisplayFormatter::new(config_clone, visual_opts)?;
    formatter.display(username, &local_data, &local_data)?;

    return Ok(());
  }

  // Create fetcher
  let provider = config_manager
    .get_provider()
    .ok_or_else(|| anyhow::anyhow!("Provider not configured"))?;
  let provider_url = config_manager
    .get_provider_url()
    .unwrap_or("https://api.github.com");
  let token = config_manager.get_token();

  // Get cache expiry from cloned config
  let cache_expiry = config_clone.cache_expiry_minutes;

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
  let cache_manager = CacheManager::new(cache_expiry)?;

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
        // Try stale cache for immediate display
        match (
          cache_manager.get_stale_cached_user_data(&username)?,
          cache_manager.get_stale_cached_stats(&username)?,
        ) {
          (Some(stale_user), Some(stale_stats)) => {
            // Display stale data immediately
            let formatter = DisplayFormatter::new(config_clone.clone(), visual_opts)?;
            formatter.display(&username, &stale_user, &stale_stats)?;

            // Spawn background refresh
            let username_clone = username.clone();
            let provider_clone = provider.to_string();
            let provider_url_clone = provider_url.to_string();
            let token_clone = token.map(|s| s.to_string());

            tokio::spawn(async move {
              if let Ok(fetcher) = fetcher::create_fetcher(
                &provider_clone,
                &provider_url_clone,
                token_clone.as_deref(),
              ) {
                if let Ok(user_data) = fetcher.fetch_user_data(&username_clone).await {
                  if let Ok(stats) = fetcher
                    .fetch_user_stats(&username_clone, Some(&user_data))
                    .await
                  {
                    if let Ok(cache) = CacheManager::new(cache_expiry) {
                      let _ = cache.cache_user_data(&username_clone, &user_data, &stats);
                    }
                  }
                }
              }
            });

            return Ok(());
          }
          _ => {
            // No cache at all, fetch fresh
            let user_data = fetcher.fetch_user_data(&username).await?;
            let stats = fetcher
              .fetch_user_stats(&username, Some(&user_data))
              .await?;
            cache_manager.cache_user_data(&username, &user_data, &stats)?;
            (user_data, stats)
          }
        }
      }
    }
  };

  // Display
  let formatter = DisplayFormatter::new(config_clone, visual_opts)?;
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

  match fetcher.get_authenticated_user().await {
    Ok(username) => {
      println!("Using authenticated user: {}", username);
      config_manager.set_default_username(username);
    }
    Err(e) => {
      eprintln!("Could not get authenticated user: {}", e);
      if provider == "github" {
        eprintln!("Please authenticate with: gh auth login");
      } else if provider == "gitlab" {
        eprintln!("Please authenticate with: glab auth login");
      } else {
        eprintln!("Please ensure you have a valid token configured");
      }
      return Err(e);
    }
  }

  let cache_expiry = interactive::prompt_cache_expiry()?;
  config_manager.config.cache_expiry_minutes = cache_expiry as u32;

  config_manager.save()?;

  Ok(())
}
