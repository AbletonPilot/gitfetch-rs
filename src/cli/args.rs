use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "gitfetch")]
#[command(about = "Neofetch-style CLI tool for git providers", long_about = None)]
#[command(version)]
pub struct Cli {
  /// Username to fetch stats for
  pub username: Option<String>,

  /// Bypass cache and fetch fresh data
  #[arg(long)]
  pub no_cache: bool,

  /// Clear the cache and exit
  #[arg(long)]
  pub clear_cache: bool,

  /// Show version information
  #[arg(long, short = 'V')]
  pub version: bool,

  /// Change the configured git provider
  #[arg(long)]
  pub change_provider: bool,

  /// Custom character for contribution blocks
  #[arg(long)]
  pub custom_box: Option<String>,

  /// Hide month/date labels
  #[arg(long)]
  pub no_date: bool,

  /// Show only the contribution graph
  #[arg(long)]
  pub graph_only: bool,

  /// Enable spaced layout
  #[arg(long)]
  pub spaced: bool,

  /// Disable spaced layout
  #[arg(long)]
  pub not_spaced: bool,

  /// Custom width for contribution graph
  #[arg(long)]
  pub width: Option<usize>,

  /// Custom height for contribution graph
  #[arg(long)]
  pub height: Option<usize>,

  /// Hide achievements section
  #[arg(long)]
  pub no_achievements: bool,

  /// Hide languages section
  #[arg(long)]
  pub no_languages: bool,

  /// Hide issues section
  #[arg(long)]
  pub no_issues: bool,

  /// Hide pull requests section
  #[arg(long)]
  pub no_pr: bool,

  /// Hide account information
  #[arg(long)]
  pub no_account: bool,

  /// Hide contribution grid/graph
  #[arg(long)]
  pub no_grid: bool,

  /// Simulate contribution graph with text (A-Z and space only)
  #[arg(long)]
  pub text: Option<String>,

  /// Simulate contribution graph with predefined shapes
  #[arg(long, value_delimiter = ',')]
  pub shape: Option<Vec<String>>,

  /// Show git timeline graph instead of contribution graph
  #[arg(long)]
  pub graph_timeline: bool,

  /// Analyze local git repository (requires .git folder)
  #[arg(long)]
  pub local: bool,
}
