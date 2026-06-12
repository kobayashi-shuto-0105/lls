use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Explore a directory with LLM-friendly annotations.
#[derive(Parser, Debug)]
#[command(name = "lls", version, about)]
pub struct CliArgs {
    /// Target path (default: `.`)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output mode: compact JSON (default)
    #[arg(long, conflicts_with_all = &["human", "long"])]
    pub json: bool,

    /// Human-readable output
    #[arg(short = 'H', long, conflicts_with_all = &["json", "long"])]
    pub human: bool,

    /// Long listing format
    #[arg(short = 'l', long = "long", conflicts_with_all = &["json", "human"])]
    pub long: bool,

    /// Scan depth (0..=8, default: 1)
    #[arg(long, default_value_t = 1)]
    pub depth: u8,

    /// Sort order
    #[arg(long, value_enum)]
    pub sort: Option<SortBy>,

    /// Explicit config path
    #[arg(long, conflicts_with = "no_config")]
    pub config: Option<PathBuf>,

    /// Skip config lookup, use built-in defaults
    #[arg(long, conflicts_with = "config")]
    pub no_config: bool,

    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Generate .lls/config.json
    Setup {
        /// Overwrite existing config
        #[arg(long)]
        force: bool,

        /// Skip confirmation
        #[arg(long)]
        yes: bool,

        /// Skip Codex, use built-in defaults
        #[arg(long)]
        without_codex: bool,
    },
}

/// Sort order for entries.
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq)]
pub enum SortBy {
    Priority,
    Name,
    Mtime,
    Size,
}

impl CliArgs {
    /// Validates CLI argument constraints beyond what clap can express.
    pub fn validate(&self) -> Result<(), String> {
        if self.depth > 8 {
            return Err("depth must be between 0 and 8".into());
        }
        if self.command.is_some() && self.long {
            return Err("--long cannot be combined with subcommands".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_path_is_dot() {
        let args = CliArgs::try_parse_from(["lls"]).unwrap();
        assert_eq!(args.path, PathBuf::from("."));
    }

    #[test]
    fn test_explicit_path() {
        let args = CliArgs::try_parse_from(["lls", "src"]).unwrap();
        assert_eq!(args.path, PathBuf::from("src"));
    }

    #[test]
    fn test_json_flag() {
        let args = CliArgs::try_parse_from(["lls", "--json"]).unwrap();
        assert!(args.json);
    }

    #[test]
    fn test_human_flag() {
        let args = CliArgs::try_parse_from(["lls", "-H"]).unwrap();
        assert!(args.human);
    }

    #[test]
    fn test_human_long_flag() {
        let args = CliArgs::try_parse_from(["lls", "--human"]).unwrap();
        assert!(args.human);
    }

    #[test]
    fn test_long_flag() {
        let args = CliArgs::try_parse_from(["lls", "-l"]).unwrap();
        assert!(args.long);
    }

    #[test]
    fn test_depth() {
        let args = CliArgs::try_parse_from(["lls", "--depth", "3"]).unwrap();
        assert_eq!(args.depth, 3);
    }

    #[test]
    fn test_config() {
        let args = CliArgs::try_parse_from(["lls", "--config", "myconfig.json"]).unwrap();
        assert_eq!(args.config, Some(PathBuf::from("myconfig.json")));
    }

    #[test]
    fn test_no_config() {
        let args = CliArgs::try_parse_from(["lls", "--no-config"]).unwrap();
        assert!(args.no_config);
    }

    #[test]
    fn test_setup_subcommand() {
        let args = CliArgs::try_parse_from(["lls", "setup"]).unwrap();
        assert!(args.command.is_some());
    }

    #[test]
    fn test_setup_with_flags() {
        let args = CliArgs::try_parse_from(["lls", "setup", "--force", "--yes", "--without-codex"])
            .unwrap();
        match &args.command {
            Some(CliCommand::Setup {
                force,
                yes,
                without_codex,
            }) => {
                assert!(*force);
                assert!(*yes);
                assert!(*without_codex);
            }
            _ => panic!("expected Setup subcommand"),
        }
    }

    #[test]
    fn test_conflicting_output_modes() {
        let result = CliArgs::try_parse_from(["lls", "--json", "--human"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_and_no_config_conflict() {
        let result = CliArgs::try_parse_from(["lls", "--config", "c.json", "--no-config"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_depth_too_high() {
        let mut args = CliArgs::try_parse_from(["lls"]).unwrap();
        args.depth = 9;
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_depth_ok() {
        let args = CliArgs::try_parse_from(["lls", "--depth", "8"]).unwrap();
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_sort_parser() {
        let args = CliArgs::try_parse_from(["lls", "--sort", "name"]).unwrap();
        assert_eq!(args.sort, Some(SortBy::Name));
        let args = CliArgs::try_parse_from(["lls", "--sort", "priority"]).unwrap();
        assert_eq!(args.sort, Some(SortBy::Priority));
        let args = CliArgs::try_parse_from(["lls", "--sort", "mtime"]).unwrap();
        assert_eq!(args.sort, Some(SortBy::Mtime));
        let args = CliArgs::try_parse_from(["lls", "--sort", "size"]).unwrap();
        assert_eq!(args.sort, Some(SortBy::Size));
    }
}
