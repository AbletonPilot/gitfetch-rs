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

  /// Change the configured git provider
  #[arg(long)]
  pub change_provider: bool,

  /// Custom character for contribution blocks
  #[arg(long)]
  pub custom_box: Option<String>,

  /// Hide month/date labels
  #[arg(long)]
  pub no_date: bool,
}
