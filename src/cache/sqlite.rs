use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, params};

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

    conn.execute(
      "CREATE INDEX IF NOT EXISTS idx_cached_at ON users(cached_at)",
      [],
    )?;

    Ok(Self {
      conn,
      cache_expiry_minutes: cache_expiry_minutes as i64,
    })
  }

  pub fn get_cached_user_data(&self, username: &str) -> Result<Option<serde_json::Value>> {
    let mut stmt = self
      .conn
      .prepare("SELECT user_data, cached_at FROM users WHERE username = ?")?;

    let result = stmt.query_row(params![username], |row| {
      let user_data: String = row.get(0)?;
      let cached_at: String = row.get(1)?;
      Ok((user_data, cached_at))
    });

    match result {
      Ok((user_data, cached_at)) => {
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
      .prepare("SELECT stats_data, cached_at FROM users WHERE username = ?")?;

    let result = stmt.query_row(params![username], |row| {
      let stats_data: String = row.get(0)?;
      let cached_at: String = row.get(1)?;
      Ok((stats_data, cached_at))
    });

    match result {
      Ok((stats_data, cached_at)) => {
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
      "INSERT OR REPLACE INTO users (username, user_data, stats_data, cached_at)
             VALUES (?1, ?2, ?3, ?4)",
      params![username, user_data_str, stats_str, now],
    )?;

    Ok(())
  }

  fn is_cache_expired(&self, cached_at: &str) -> Result<bool> {
    let cached_time = DateTime::parse_from_rfc3339(cached_at)?;
    let expiry_time = Utc::now() - Duration::minutes(self.cache_expiry_minutes);
    Ok(cached_time < expiry_time)
  }

  pub fn clear(&self) -> Result<()> {
    self.conn.execute("DELETE FROM users", [])?;
    Ok(())
  }
}
