use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};

const CACHE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct CacheManager {
  conn: Connection,
  cache_expiry_minutes: i64,
}

impl CacheManager {
  pub fn new(cache_expiry_minutes: u32) -> Result<Self> {
    let project_dirs = directories::ProjectDirs::from("com", "gitfetch", "gitfetch")
      .ok_or_else(|| anyhow::anyhow!("Could not determine cache directory"))?;

    let cache_dir = project_dirs.data_local_dir();
    std::fs::create_dir_all(cache_dir)?;

    let db_path = cache_dir.join("cache.db");
    let conn = Connection::open(db_path)?;

    conn.execute(
      "CREATE TABLE IF NOT EXISTS users (
                username TEXT PRIMARY KEY,
                user_data TEXT NOT NULL,
                stats_data TEXT NOT NULL,
                cached_at TEXT NOT NULL
            )",
      [],
    )?;

    // Add version column if it doesn't exist (for migration)
    let _ = conn.execute(
      "ALTER TABLE users ADD COLUMN version TEXT NOT NULL DEFAULT '0.0.0'",
      [],
    );

    conn.execute(
      "CREATE INDEX IF NOT EXISTS idx_cached_at ON users(cached_at)",
      [],
    )?;

    // Clear cache if version changed
    conn.execute(
      "DELETE FROM users WHERE version != ?",
      params![CACHE_VERSION],
    )?;

    Ok(Self {
      conn,
      cache_expiry_minutes: cache_expiry_minutes as i64,
    })
  }

  pub fn get_cached_user_data(&self, username: &str) -> Result<Option<serde_json::Value>> {
    let mut stmt = self
      .conn
      .prepare("SELECT user_data, cached_at, version FROM users WHERE username = ?")?;

    let result = stmt.query_row(params![username], |row| {
      let user_data: String = row.get(0)?;
      let cached_at: String = row.get(1)?;
      let version: String = row.get(2)?;
      Ok((user_data, cached_at, version))
    });

    match result {
      Ok((user_data, cached_at, version)) => {
        // Check version match
        if version != CACHE_VERSION {
          return Ok(None);
        }
        if self.is_cache_expired(&cached_at)? {
          return Ok(None);
        }
        let data = serde_json::from_str(&user_data)?;
        Ok(Some(data))
      }
      Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
      Err(e) => Err(e.into()),
    }
  }

  pub fn get_cached_stats(&self, username: &str) -> Result<Option<serde_json::Value>> {
    let mut stmt = self
      .conn
      .prepare("SELECT stats_data, cached_at, version FROM users WHERE username = ?")?;

    let result = stmt.query_row(params![username], |row| {
      let stats_data: String = row.get(0)?;
      let cached_at: String = row.get(1)?;
      let version: String = row.get(2)?;
      Ok((stats_data, cached_at, version))
    });

    match result {
      Ok((stats_data, cached_at, version)) => {
        // Check version match
        if version != CACHE_VERSION {
          return Ok(None);
        }
        if self.is_cache_expired(&cached_at)? {
          return Ok(None);
        }
        let data = serde_json::from_str(&stats_data)?;
        Ok(Some(data))
      }
      Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
      Err(e) => Err(e.into()),
    }
  }

  pub fn cache_user_data(
    &self,
    username: &str,
    user_data: &serde_json::Value,
    stats: &serde_json::Value,
  ) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let user_data_str = serde_json::to_string(user_data)?;
    let stats_str = serde_json::to_string(stats)?;

    self.conn.execute(
      "INSERT OR REPLACE INTO users (username, user_data, stats_data, cached_at, version)
             VALUES (?1, ?2, ?3, ?4, ?5)",
      params![username, user_data_str, stats_str, now, CACHE_VERSION],
    )?;

    Ok(())
  }

  fn is_cache_expired(&self, cached_at: &str) -> Result<bool> {
    let cached_time = DateTime::parse_from_rfc3339(cached_at)?;
    let expiry_time = Utc::now() - Duration::minutes(self.cache_expiry_minutes);
    Ok(cached_time < expiry_time)
  }

  // Get stale cache (ignore expiry, only check version)
  pub fn get_stale_cached_user_data(&self, username: &str) -> Result<Option<serde_json::Value>> {
    let mut stmt = self
      .conn
      .prepare("SELECT user_data, version FROM users WHERE username = ?")?;

    let result = stmt.query_row(params![username], |row| {
      let user_data: String = row.get(0)?;
      let version: String = row.get(1)?;
      Ok((user_data, version))
    });

    match result {
      Ok((user_data, version)) => {
        // Only check version match, ignore expiry
        if version != CACHE_VERSION {
          return Ok(None);
        }
        let data = serde_json::from_str(&user_data)?;
        Ok(Some(data))
      }
      Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
      Err(e) => Err(e.into()),
    }
  }

  pub fn get_stale_cached_stats(&self, username: &str) -> Result<Option<serde_json::Value>> {
    let mut stmt = self
      .conn
      .prepare("SELECT stats_data, version FROM users WHERE username = ?")?;

    let result = stmt.query_row(params![username], |row| {
      let stats_data: String = row.get(0)?;
      let version: String = row.get(1)?;
      Ok((stats_data, version))
    });

    match result {
      Ok((stats_data, version)) => {
        // Only check version match, ignore expiry
        if version != CACHE_VERSION {
          return Ok(None);
        }
        let data = serde_json::from_str(&stats_data)?;
        Ok(Some(data))
      }
      Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
      Err(e) => Err(e.into()),
    }
  }

  pub fn clear(&self) -> Result<()> {
    self.conn.execute("DELETE FROM users", [])?;
    Ok(())
  }
}
