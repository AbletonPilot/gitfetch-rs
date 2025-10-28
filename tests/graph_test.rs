use gitfetch_rs::config::ColorConfig;
use gitfetch_rs::display::graph::ContributionGraph;
use serde_json::json;

#[test]
fn test_contribution_graph_from_json() {
  let data = json!([
      {
          "contributionDays": [
              {"contributionCount": 0, "date": "2024-01-01"},
              {"contributionCount": 5, "date": "2024-01-02"},
              {"contributionCount": 10, "date": "2024-01-03"},
          ]
      }
  ]);

  let graph = ContributionGraph::from_json(&data);
  let lines = graph.render(None, None, "■", &Default::default(), false, false);
  assert!(!lines.is_empty());
}

#[test]
fn test_contribution_graph_from_grid() {
  let grid = vec![
    vec![0, 1, 2, 3, 4],
    vec![1, 2, 3, 4, 0],
    vec![2, 3, 4, 0, 1],
    vec![3, 4, 0, 1, 2],
    vec![4, 0, 1, 2, 3],
    vec![0, 1, 2, 3, 4],
    vec![1, 2, 3, 4, 0],
  ];

  let graph = ContributionGraph::from_grid(grid);
  let lines = graph.render(None, None, "■", &ColorConfig::default(), false, false);
  assert!(!lines.is_empty());
}

#[test]
fn test_contribution_graph_empty() {
  let data = json!([]);
  let _graph = ContributionGraph::from_json(&data);
  // Empty graph should not panic
  assert!(true);
}

#[test]
fn test_contribution_graph_from_empty_grid() {
  let grid: Vec<Vec<u8>> = vec![];
  let _graph = ContributionGraph::from_grid(grid);
  // Empty grid should not panic
  assert!(true);
}
