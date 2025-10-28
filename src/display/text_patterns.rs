use std::collections::HashMap;

pub type Pattern = Vec<Vec<u8>>;

pub fn get_patterns() -> HashMap<String, Pattern> {
  let mut patterns = HashMap::new();

  // A-Z alphabet patterns
  patterns.insert(
    "A".to_string(),
    vec![
      vec![0, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 3, 3, 3, 3, 3, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
    ],
  );

  patterns.insert(
    "B".to_string(),
    vec![
      vec![3, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 3, 3, 3, 3, 3, 0],
    ],
  );

  patterns.insert(
    "C".to_string(),
    vec![
      vec![0, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![0, 3, 3, 3, 3, 3, 0],
    ],
  );

  patterns.insert(
    "D".to_string(),
    vec![
      vec![3, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 3, 3, 3, 3, 3, 0],
    ],
  );

  patterns.insert(
    "E".to_string(),
    vec![
      vec![3, 3, 3, 3, 3, 3, 3],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 3, 3, 3, 3, 3, 3],
    ],
  );

  patterns.insert(
    "F".to_string(),
    vec![
      vec![3, 3, 3, 3, 3, 3, 3],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
    ],
  );

  patterns.insert(
    "G".to_string(),
    vec![
      vec![0, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 3, 3, 3, 3, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![0, 3, 3, 3, 3, 3, 0],
    ],
  );

  patterns.insert(
    "H".to_string(),
    vec![
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 3, 3, 3, 3, 3, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
    ],
  );

  patterns.insert(
    "I".to_string(),
    vec![
      vec![3, 3, 3, 3, 3, 3, 3],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![3, 3, 3, 3, 3, 3, 3],
    ],
  );

  patterns.insert(
    "J".to_string(),
    vec![
      vec![3, 3, 3, 3, 3, 3, 3],
      vec![0, 0, 0, 0, 0, 3, 0],
      vec![0, 0, 0, 0, 0, 3, 0],
      vec![0, 0, 0, 0, 0, 3, 0],
      vec![0, 0, 0, 0, 0, 3, 0],
      vec![3, 0, 0, 0, 0, 3, 0],
      vec![0, 3, 3, 3, 3, 0, 0],
    ],
  );

  patterns.insert(
    "K".to_string(),
    vec![
      vec![3, 0, 0, 0, 0, 3, 0],
      vec![3, 0, 0, 0, 3, 0, 0],
      vec![3, 0, 0, 3, 0, 0, 0],
      vec![3, 3, 3, 0, 0, 0, 0],
      vec![3, 0, 0, 3, 0, 0, 0],
      vec![3, 0, 0, 0, 3, 0, 0],
      vec![3, 0, 0, 0, 0, 3, 0],
    ],
  );

  patterns.insert(
    "L".to_string(),
    vec![
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 3, 3, 3, 3, 3, 3],
    ],
  );

  patterns.insert(
    "M".to_string(),
    vec![
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 3, 0, 0, 0, 3, 3],
      vec![3, 0, 3, 0, 3, 0, 3],
      vec![3, 0, 0, 3, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
    ],
  );

  patterns.insert(
    "N".to_string(),
    vec![
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 3, 0, 0, 0, 0, 3],
      vec![3, 0, 3, 0, 0, 0, 3],
      vec![3, 0, 0, 3, 0, 0, 3],
      vec![3, 0, 0, 0, 3, 0, 3],
      vec![3, 0, 0, 0, 0, 3, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
    ],
  );

  patterns.insert(
    "O".to_string(),
    vec![
      vec![0, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![0, 3, 3, 3, 3, 3, 0],
    ],
  );

  patterns.insert(
    "P".to_string(),
    vec![
      vec![3, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
    ],
  );

  patterns.insert(
    "Q".to_string(),
    vec![
      vec![0, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 3, 0, 3],
      vec![3, 0, 0, 0, 0, 3, 0],
      vec![0, 3, 3, 3, 3, 0, 3],
    ],
  );

  patterns.insert(
    "R".to_string(),
    vec![
      vec![3, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 3, 3, 3, 3, 3, 0],
      vec![3, 0, 0, 3, 0, 0, 0],
      vec![3, 0, 0, 0, 3, 0, 0],
      vec![3, 0, 0, 0, 0, 3, 0],
    ],
  );

  patterns.insert(
    "S".to_string(),
    vec![
      vec![0, 3, 3, 3, 3, 3, 3],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![3, 0, 0, 0, 0, 0, 0],
      vec![0, 3, 3, 3, 3, 3, 0],
      vec![0, 0, 0, 0, 0, 0, 3],
      vec![0, 0, 0, 0, 0, 0, 3],
      vec![3, 3, 3, 3, 3, 3, 0],
    ],
  );

  patterns.insert(
    "T".to_string(),
    vec![
      vec![3, 3, 3, 3, 3, 3, 3],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
    ],
  );

  patterns.insert(
    "U".to_string(),
    vec![
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![0, 3, 3, 3, 3, 3, 0],
    ],
  );

  patterns.insert(
    "V".to_string(),
    vec![
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![0, 3, 0, 0, 0, 3, 0],
      vec![0, 0, 3, 3, 3, 0, 0],
    ],
  );

  patterns.insert(
    "W".to_string(),
    vec![
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![3, 0, 0, 3, 0, 0, 3],
      vec![3, 0, 3, 0, 3, 0, 3],
      vec![3, 3, 0, 0, 0, 3, 3],
      vec![3, 0, 0, 0, 0, 0, 3],
    ],
  );

  patterns.insert(
    "X".to_string(),
    vec![
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![0, 3, 0, 0, 0, 3, 0],
      vec![0, 0, 3, 0, 3, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 3, 0, 3, 0, 0],
      vec![0, 3, 0, 0, 0, 3, 0],
      vec![3, 0, 0, 0, 0, 0, 3],
    ],
  );

  patterns.insert(
    "Y".to_string(),
    vec![
      vec![3, 0, 0, 0, 0, 0, 3],
      vec![0, 3, 0, 0, 0, 3, 0],
      vec![0, 0, 3, 0, 3, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
    ],
  );

  patterns.insert(
    "Z".to_string(),
    vec![
      vec![3, 3, 3, 3, 3, 3, 3],
      vec![0, 0, 0, 0, 0, 3, 0],
      vec![0, 0, 0, 0, 3, 0, 0],
      vec![0, 0, 0, 3, 0, 0, 0],
      vec![0, 0, 3, 0, 0, 0, 0],
      vec![0, 3, 0, 0, 0, 0, 0],
      vec![3, 3, 3, 3, 3, 3, 3],
    ],
  );

  patterns.insert(
    " ".to_string(),
    vec![
      vec![0, 0, 0, 0, 0, 0, 0],
      vec![0, 0, 0, 0, 0, 0, 0],
      vec![0, 0, 0, 0, 0, 0, 0],
      vec![0, 0, 0, 0, 0, 0, 0],
      vec![0, 0, 0, 0, 0, 0, 0],
      vec![0, 0, 0, 0, 0, 0, 0],
      vec![0, 0, 0, 0, 0, 0, 0],
    ],
  );

  // Shape patterns
  patterns.insert(
    "heart".to_string(),
    vec![
      vec![0, 4, 4, 0, 4, 4, 0],
      vec![4, 2, 2, 4, 2, 2, 4],
      vec![4, 2, 2, 2, 2, 2, 4],
      vec![4, 2, 2, 2, 2, 2, 4],
      vec![0, 4, 2, 2, 2, 4, 0],
      vec![0, 0, 4, 2, 4, 0, 0],
      vec![0, 0, 0, 4, 0, 0, 0],
    ],
  );

  patterns.insert(
    "octocat".to_string(),
    vec![
      vec![0, 0, 0, 4, 0, 0, 0, 4, 0],
      vec![0, 0, 4, 4, 4, 4, 4, 4, 4],
      vec![0, 0, 4, 1, 3, 3, 3, 1, 4],
      vec![0, 0, 4, 3, 3, 3, 3, 3, 4],
      vec![4, 0, 3, 4, 3, 3, 3, 4, 3],
      vec![0, 4, 0, 0, 4, 4, 4, 0, 0],
      vec![0, 0, 4, 4, 4, 4, 4, 4, 4],
    ],
  );

  patterns
}

pub fn text_to_grid(text: &str) -> Result<Vec<Vec<u8>>, String> {
  if text.is_empty() {
    return Ok(Vec::new());
  }

  let patterns = get_patterns();
  let mut grid: Vec<Vec<u8>> = Vec::new();

  for ch in text.to_uppercase().chars() {
    if !ch.is_ascii_alphabetic() && ch != ' ' {
      return Err(format!(
        "Text mode only supports A-Z and space. Use --shape for predefined shapes. Got: '{}'",
        ch
      ));
    }

    let pattern = patterns
      .get(&ch.to_string())
      .or_else(|| patterns.get(" "))
      .ok_or_else(|| format!("Pattern not found for '{}'", ch))?;

    if grid.is_empty() {
      grid = pattern.clone();
    } else {
      for (i, row) in pattern.iter().enumerate() {
        grid[i].extend(row);
      }
    }

    // Add one-column spacer after each character
    for row in grid.iter_mut() {
      row.push(0);
    }
  }

  Ok(grid)
}

pub fn shape_to_grid(shapes: &[String]) -> Result<Vec<Vec<u8>>, String> {
  if shapes.is_empty() {
    return Ok(Vec::new());
  }

  let patterns = get_patterns();
  let mut grid: Vec<Vec<u8>> = Vec::new();

  for shape_name in shapes {
    let pattern = patterns
      .get(shape_name)
      .ok_or_else(|| format!("Unknown shape: {}", shape_name))?;

    if grid.is_empty() {
      grid = pattern.clone();
    } else {
      // Add spacer column
      for row in grid.iter_mut() {
        row.push(0);
      }

      // Append shape
      for (i, row) in pattern.iter().enumerate() {
        grid[i].extend(row);
      }
    }
  }

  Ok(grid)
}
