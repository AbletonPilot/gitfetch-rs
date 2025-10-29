use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "gitfetch-rs")]
#[command(
  about = "A neofetch-style CLI tool for git.\nOriginal Python CLI is https://github.com/Matars/gitfetch\nSupports GitHub, GitLab, Gitea, and Sourcehut."
)]
#[command(version)]
pub struct Cli {
  /// Username to fetch stats for
  pub username: Option<String>,

  // ===== General Options =====
  /// Bypass cache and fetch fresh data
  #[arg(long, help_heading = "General Options")]
  pub no_cache: bool,

  /// Clear the cache and exit
  #[arg(long, help_heading = "General Options")]
  pub clear_cache: bool,

  /// Show version and check for updates
  #[arg(long, short = 'V', help_heading = "General Options")]
  pub version: bool,

  /// Change the configured git provider
  #[arg(long, help_heading = "General Options")]
  pub change_provider: bool,

  /// Fetch data specific to current local repo (requires .git folder)
  #[arg(long, help_heading = "General Options")]
  pub local: bool,

  // ===== Visual Options =====
  /// Enable spaced layout
  #[arg(long, help_heading = "Visual Options")]
  pub spaced: bool,

  /// Disable spaced layout
  #[arg(long, help_heading = "Visual Options")]
  pub not_spaced: bool,

  /// Custom character for contribution blocks (e.g., '■', '█')
  #[arg(long, help_heading = "Visual Options")]
  pub custom_box: Option<String>,

  /// Show only the contribution graph
  #[arg(long, help_heading = "Visual Options")]
  pub graph_only: bool,

  /// Set custom width for contribution graph
  #[arg(long, help_heading = "Visual Options")]
  pub width: Option<usize>,

  /// Set custom height for contribution graph
  #[arg(long, help_heading = "Visual Options")]
  pub height: Option<usize>,

  /// Display text as contribution graph pattern (simulation only)
  #[arg(long, help_heading = "Visual Options")]
  pub text: Option<String>,

  /// Display one or more predefined shapes as contribution graph (simulation only). Provide multiple shapes: --shape kitty,kitty
  #[arg(long, value_delimiter = ',', help_heading = "Visual Options")]
  pub shape: Option<Vec<String>>,

  /// Show git timeline graph instead of contribution graph
  #[arg(long, help_heading = "Visual Options")]
  pub graph_timeline: bool,

  // ===== Visibility =====
  /// Hide month/date labels on contribution graph
  #[arg(long, help_heading = "Visibility")]
  pub no_date: bool,

  /// Hide achievements section
  #[arg(long, help_heading = "Visibility")]
  pub no_achievements: bool,

  /// Hide languages section
  #[arg(long, help_heading = "Visibility")]
  pub no_languages: bool,

  /// Hide issues section
  #[arg(long, help_heading = "Visibility")]
  pub no_issues: bool,

  /// Hide pull requests section
  #[arg(long, help_heading = "Visibility")]
  pub no_pr: bool,

  /// Hide account information section
  #[arg(long, help_heading = "Visibility")]
  pub no_account: bool,

  /// Hide contribution grid/graph
  #[arg(long, help_heading = "Visibility")]
  pub no_grid: bool,
}
