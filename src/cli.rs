use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "lls",
    about = "ls for LLMs — explore directories with semantic classification"
)]
pub struct Cli {
    /// Target path to scan (directory or file)
    #[arg(default_value = ".")]
    pub path: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,

    /// Compact JSON output (no extra whitespace)
    #[arg(long)]
    pub compact: bool,

    /// Maximum depth to scan (0 = target only, 1 = direct children, etc.)
    #[arg(long, default_value = "1")]
    pub depth: usize,
}
