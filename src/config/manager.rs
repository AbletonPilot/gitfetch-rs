use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
  pub provider: Option<String>,
  pub provider_url: Option<String>,
  pub token: Option<String>,
  pub default_username: Option<String>,
  pub cache_expiry_minutes: u32,
  pub custom_box: Option<String>,
  #[serde(default = "default_show_date")]
  pub show_date: bool,
  pub colors: ColorConfig,
}

fn default_show_date() -> bool {
  true
}

impl Default for Config {
  fn default() -> Self {
    Self {
      provider: None,
      provider_url: None,
      token: None,
      default_username: None,
      cache_expiry_minutes: 15,
      custom_box: None,
      show_date: true,
      colors: ColorConfig::default(),
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorConfig {
  pub level_0: String,
  pub level_1: String,
  pub level_2: String,
  pub level_3: String,
  pub level_4: String,
}

impl Default for ColorConfig {
  fn default() -> Self {
    Self {
      level_0: "#ebedf0".to_string(), // Light gray like Python
      level_1: "#9be9a8".to_string(),
      level_2: "#40c463".to_string(),
      level_3: "#30a14e".to_string(),
      level_4: "#216e39".to_string(),
    }
  }
}

pub struct ConfigManager {
  config_path: PathBuf,
  pub config: Config,
}

impl ConfigManager {
  pub fn new() -> Result<Self> {
    let project_dirs = ProjectDirs::from("com", "gitfetch", "gitfetch-rs")
      .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let config_dir = project_dirs.config_dir();
    std::fs::create_dir_all(config_dir)?;

    let config_path = config_dir.join("config.toml");

    let config = if config_path.exists() {
      let content = std::fs::read_to_string(&config_path)?;
      toml::from_str(&content).unwrap_or_else(|_| Config::default())
    } else {
      // Use defaults if no config file exists (like Python version)
      Config::default()
    };

    Ok(Self {
      config_path,
      config,
    })
  }

  pub fn is_initialized(&self) -> bool {
    self.config.provider.is_some()
  }

  pub fn save(&self) -> Result<()> {
    let content = toml::to_string_pretty(&self.config)?;
    std::fs::write(&self.config_path, content)?;
    Ok(())
  }

  pub fn get_provider(&self) -> Option<&str> {
    self.config.provider.as_deref()
  }

  pub fn set_provider(&mut self, provider: String) {
    self.config.provider = Some(provider);
  }

  pub fn get_provider_url(&self) -> Option<&str> {
    self.config.provider_url.as_deref()
  }

  pub fn set_provider_url(&mut self, url: String) {
    self.config.provider_url = Some(url);
  }

  pub fn get_token(&self) -> Option<&str> {
    self.config.token.as_deref()
  }

  pub fn set_token(&mut self, token: String) {
    self.config.token = Some(token);
  }

  pub fn get_default_username(&self) -> Option<&str> {
    self.config.default_username.as_deref()
  }

  pub fn set_default_username(&mut self, username: String) {
    self.config.default_username = Some(username);
  }
}
