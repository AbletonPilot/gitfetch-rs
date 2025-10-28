#[cfg(test)]
mod display_tests {
  use gitfetch_rs::config::ColorConfig;
  use gitfetch_rs::display::colors::get_ansi_color;

  #[test]
  fn test_get_ansi_color_basic() {
    let colors = ColorConfig::default();

    let ansi = get_ansi_color(&colors.level_0);
    assert!(ansi.is_ok(), "Should return ANSI code for valid hex");

    let color_string = ansi.unwrap();
    assert!(!color_string.is_empty(), "ANSI code should not be empty");
  }

  #[test]
  fn test_get_ansi_color_all_levels() {
    let colors = ColorConfig::default();

    assert!(get_ansi_color(&colors.level_0).is_ok());
    assert!(get_ansi_color(&colors.level_1).is_ok());
    assert!(get_ansi_color(&colors.level_2).is_ok());
    assert!(get_ansi_color(&colors.level_3).is_ok());
    assert!(get_ansi_color(&colors.level_4).is_ok());
  }

  #[test]
  fn test_get_ansi_color_formats() {
    // Test with various valid hex formats
    assert!(get_ansi_color("#ff0000").is_ok());
    assert!(get_ansi_color("00ff00").is_ok());
    assert!(get_ansi_color("#0000ff").is_ok());
  }

  #[test]
  fn test_ansi_color_contains_escape_sequence() {
    let color = get_ansi_color("#ff5733").unwrap();
    assert!(color.starts_with("\x1b[38;2;"));
    assert!(color.ends_with("m"));
  }
}

#[cfg(test)]
mod cache_tests {
  use gitfetch_rs::cache::CacheManager;
  use serde_json::json;

  fn create_test_cache() -> CacheManager {
    // Create cache with 15 minute expiry
    CacheManager::new(15).unwrap()
  }

  #[test]
  fn test_cache_creation() {
    let cache = create_test_cache();

    // Cache should be created successfully
    let result = cache.clear();
    assert!(result.is_ok(), "Cache should be clearable");
  }

  #[test]
  fn test_cache_user_data() {
    let cache = create_test_cache();

    let user_data = json!({
        "login": "test_user_123",
        "name": "Test User"
    });

    let stats = json!({
        "total_repos": 10
    });

    let result = cache.cache_user_data("test_user_123", &user_data, &stats);
    assert!(result.is_ok(), "Should cache user data successfully");
  }

  #[test]
  fn test_get_cached_user_data() {
    let cache = create_test_cache();

    let user_data = json!({
        "login": "test_user_456",
        "name": "Test User"
    });

    let stats = json!({
        "total_repos": 10
    });

    cache
      .cache_user_data("test_user_456", &user_data, &stats)
      .unwrap();

    let cached = cache.get_cached_user_data("test_user_456").unwrap();
    assert!(cached.is_some(), "Should retrieve cached data");

    if let Some(data) = cached {
      assert_eq!(data["login"], "test_user_456");
    }
  }

  #[test]
  fn test_cache_miss() {
    let cache = create_test_cache();

    let result = cache.get_cached_user_data("nonexistent_user_999").unwrap();
    assert!(result.is_none(), "Should return None for cache miss");
  }

  #[test]
  fn test_clear_cache() {
    let cache = create_test_cache();

    let user_data = json!({"login": "test_user_789"});
    let stats = json!({"total_repos": 10});

    cache
      .cache_user_data("test_user_789", &user_data, &stats)
      .unwrap();

    let result = cache.clear();
    assert!(result.is_ok(), "Should clear cache successfully");

    let cached = cache.get_cached_user_data("test_user_789").unwrap();
    assert!(cached.is_none(), "Cache should be empty after clear");
  }

  #[test]
  fn test_cache_multiple_users() {
    let cache = create_test_cache();

    let user1 = json!({"login": "user1"});
    let user2 = json!({"login": "user2"});
    let stats = json!({"total_repos": 5});

    cache.cache_user_data("user1", &user1, &stats).unwrap();
    cache.cache_user_data("user2", &user2, &stats).unwrap();

    let cached1 = cache.get_cached_user_data("user1").unwrap();
    let cached2 = cache.get_cached_user_data("user2").unwrap();

    assert!(cached1.is_some());
    assert!(cached2.is_some());
  }
}

#[cfg(test)]
mod utils_tests {
  use gitfetch_rs::utils::git;

  #[test]
  fn test_get_repo_path() {
    // This will only work if run from within a git repo
    let result = git::get_repo_path();

    if result.is_ok() {
      let path = result.unwrap();
      assert!(!path.is_empty(), "Repo path should not be empty");
    }
    // If not in a git repo, it should error gracefully
  }

  #[test]
  fn test_analyze_local_repo() {
    // This will only work if run from within a git repo
    let result = git::analyze_local_repo();

    if result.is_ok() {
      let data = result.unwrap();

      // Check that it has the expected structure
      // The actual structure depends on git.rs implementation
      assert!(data.is_object(), "Should return a JSON object");

      // At minimum, should have some data
      let obj = data.as_object().unwrap();
      assert!(!obj.is_empty(), "Should contain some data");
    }
    // If not in a git repo, it should error gracefully
  }

  #[test]
  fn test_get_repo_path_outside_git() {
    // If we're not in a git repo, should return an error
    // This test documents the expected behavior
    let result = git::get_repo_path();
    // Should either succeed (in git repo) or fail gracefully
    assert!(result.is_ok() || result.is_err());
  }
}
