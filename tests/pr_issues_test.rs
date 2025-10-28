#[cfg(test)]
mod pr_issues_tests {
  use gitfetch_rs::config::Config;
  use gitfetch_rs::display::formatter::{DisplayFormatter, VisualOptions};
  use serde_json::json;

  fn create_test_config() -> Config {
    Config::default()
  }

  #[test]
  fn test_format_pull_requests_with_data() {
    let config = create_test_config();
    let visual_opts = VisualOptions::default();
    let formatter = DisplayFormatter::new(config, visual_opts).unwrap();

    let stats = json!({
      "pull_requests": {
        "open": 5,
        "awaiting_review": 3,
        "mentions": 2
      },
      "contribution_graph": []
    });

    let user_data = json!({
      "name": "Test User",
      "login": "testuser"
    });

    // This should not panic even with PR data
    let result = formatter.display("testuser", &user_data, &stats);
    assert!(result.is_ok());
  }

  #[test]
  fn test_format_issues_with_data() {
    let config = create_test_config();
    let visual_opts = VisualOptions::default();
    let formatter = DisplayFormatter::new(config, visual_opts).unwrap();

    let stats = json!({
      "issues": {
        "assigned": 8,
        "created": 12,
        "mentions": 4
      },
      "contribution_graph": []
    });

    let user_data = json!({
      "name": "Test User",
      "login": "testuser"
    });

    // This should not panic even with issue data
    let result = formatter.display("testuser", &user_data, &stats);
    assert!(result.is_ok());
  }

  #[test]
  fn test_no_pr_option() {
    let config = create_test_config();
    let visual_opts = VisualOptions {
      no_pr: true,
      ..Default::default()
    };

    let formatter = DisplayFormatter::new(config, visual_opts).unwrap();

    let stats = json!({
      "pull_requests": {
        "open": 5,
        "awaiting_review": 3,
        "mentions": 2
      },
      "contribution_graph": []
    });

    let user_data = json!({
      "name": "Test User",
      "login": "testuser"
    });

    // Should work with no_pr flag
    let result = formatter.display("testuser", &user_data, &stats);
    assert!(result.is_ok());
  }

  #[test]
  fn test_no_issues_option() {
    let config = create_test_config();
    let visual_opts = VisualOptions {
      no_issues: true,
      ..Default::default()
    };

    let formatter = DisplayFormatter::new(config, visual_opts).unwrap();

    let stats = json!({
      "issues": {
        "assigned": 8,
        "created": 12,
        "mentions": 4
      },
      "contribution_graph": []
    });

    let user_data = json!({
      "name": "Test User",
      "login": "testuser"
    });

    // Should work with no_issues flag
    let result = formatter.display("testuser", &user_data, &stats);
    assert!(result.is_ok());
  }

  #[test]
  fn test_empty_pr_data() {
    let config = create_test_config();
    let visual_opts = VisualOptions::default();
    let formatter = DisplayFormatter::new(config, visual_opts).unwrap();

    let stats = json!({
      "pull_requests": {
        "open": 0,
        "awaiting_review": 0,
        "mentions": 0
      },
      "contribution_graph": []
    });

    let user_data = json!({
      "name": "Test User",
      "login": "testuser"
    });

    // Should handle zero values gracefully
    let result = formatter.display("testuser", &user_data, &stats);
    assert!(result.is_ok());
  }

  #[test]
  fn test_combined_pr_and_issues() {
    let config = create_test_config();
    let visual_opts = VisualOptions::default();
    let formatter = DisplayFormatter::new(config, visual_opts).unwrap();

    let stats = json!({
      "pull_requests": {
        "open": 5,
        "awaiting_review": 3,
        "mentions": 2
      },
      "issues": {
        "assigned": 8,
        "created": 12,
        "mentions": 4
      },
      "contribution_graph": []
    });

    let user_data = json!({
      "name": "Test User",
      "login": "testuser"
    });

    // Should display both sections
    let result = formatter.display("testuser", &user_data, &stats);
    assert!(result.is_ok());
  }
}
