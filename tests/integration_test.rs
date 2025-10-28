use gitfetch_rs::display::text_patterns::{shape_to_grid, text_to_grid};

#[test]
fn test_text_to_grid_simple() {
  let result = text_to_grid("A");
  assert!(result.is_ok());

  let grid = result.unwrap();
  assert_eq!(grid.len(), 7); // 7 rows
  assert!(!grid[0].is_empty()); // Has content
}

#[test]
fn test_text_to_grid_multiple_chars() {
  let result = text_to_grid("AB");
  assert!(result.is_ok());

  let grid = result.unwrap();
  assert_eq!(grid.len(), 7); // 7 rows
  // Should be wider than single character
  assert!(grid[0].len() > 7);
}

#[test]
fn test_text_to_grid_invalid_char() {
  let result = text_to_grid("A1");
  assert!(result.is_err());

  let err = result.unwrap_err();
  assert!(err.contains("Text mode only supports A-Z and space"));
}

#[test]
fn test_text_to_grid_lowercase() {
  let result = text_to_grid("abc");
  assert!(result.is_ok());

  let grid = result.unwrap();
  assert_eq!(grid.len(), 7);
}

#[test]
fn test_text_to_grid_with_space() {
  let result = text_to_grid("A B");
  assert!(result.is_ok());

  let grid = result.unwrap();
  assert_eq!(grid.len(), 7);
}

#[test]
fn test_shape_to_grid_heart() {
  let result = shape_to_grid(&vec!["heart".to_string()]);
  assert!(result.is_ok());

  let grid = result.unwrap();
  assert_eq!(grid.len(), 7); // 7 rows
  assert!(!grid[0].is_empty());
}

#[test]
fn test_shape_to_grid_octocat() {
  let result = shape_to_grid(&vec!["octocat".to_string()]);
  assert!(result.is_ok());

  let grid = result.unwrap();
  assert_eq!(grid.len(), 7);
}

#[test]
fn test_shape_to_grid_unknown() {
  let result = shape_to_grid(&vec!["unknown_shape".to_string()]);
  assert!(result.is_err());

  let err = result.unwrap_err();
  assert!(err.contains("Unknown shape"));
}

#[test]
fn test_shape_to_grid_multiple() {
  let result = shape_to_grid(&vec!["heart".to_string(), "octocat".to_string()]);
  assert!(result.is_ok());

  let grid = result.unwrap();
  assert_eq!(grid.len(), 7);
  // Should be wider than single shape
  assert!(grid[0].len() > 7);
}

#[test]
fn test_text_to_grid_empty() {
  let result = text_to_grid("");
  assert!(result.is_ok());

  let grid = result.unwrap();
  assert!(grid.is_empty());
}

#[test]
fn test_shape_to_grid_empty() {
  let result = shape_to_grid(&vec![]);
  assert!(result.is_ok());

  let grid = result.unwrap();
  assert!(grid.is_empty());
}
