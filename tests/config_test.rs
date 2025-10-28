use gitfetch_rs::config::ColorConfig;

#[test]
fn test_color_config_default() {
  let config = ColorConfig::default();

  assert!(
    !config.level_0.is_empty(),
    "Level 0 color should not be empty"
  );
  assert!(
    !config.level_1.is_empty(),
    "Level 1 color should not be empty"
  );
  assert!(
    !config.level_2.is_empty(),
    "Level 2 color should not be empty"
  );
  assert!(
    !config.level_3.is_empty(),
    "Level 3 color should not be empty"
  );
  assert!(
    !config.level_4.is_empty(),
    "Level 4 color should not be empty"
  );

  // Check hex format
  assert!(
    config.level_0.starts_with('#'),
    "Colors should start with #"
  );
  assert_eq!(config.level_0.len(), 7, "Hex colors should be 7 chars");
}

#[test]
fn test_color_config_levels_different() {
  let config = ColorConfig::default();

  // Each level should have different colors
  assert_ne!(config.level_0, config.level_1);
  assert_ne!(config.level_1, config.level_2);
  assert_ne!(config.level_2, config.level_3);
  assert_ne!(config.level_3, config.level_4);
}
