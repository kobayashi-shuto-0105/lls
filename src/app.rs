use crate::cli::{CliArgs, CliCommand, SortBy};
use crate::error::AppError;

/// Result of parsing CLI into a request.
#[derive(Debug)]
pub enum CommandRequest {
    List(ListRequest),
    Setup(SetupRequest),
}

#[derive(Debug)]
pub struct ListRequest {
    pub path: String,
    pub output_mode: OutputMode,
    pub depth: u8,
    pub sort: Option<SortBy>,
    pub config_source: ConfigSource,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputMode {
    Json,
    Human,
    Long,
}

#[derive(Debug)]
pub enum ConfigSource {
    Explicit(String),
    Auto,
    None,
}

#[derive(Debug)]
pub struct SetupRequest {
    pub force: bool,
    pub yes: bool,
    pub without_codex: bool,
    pub path: String,
}

/// Converts parsed CLI args into a validated command request.
pub fn parse_request(args: CliArgs) -> Result<CommandRequest, AppError> {
    args.validate().map_err(AppError::Cli)?;

    // Determine output mode
    let output_mode = if args.human {
        OutputMode::Human
    } else if args.long {
        OutputMode::Long
    } else {
        OutputMode::Json
    };

    // Handle subcommands
    if let Some(cmd) = args.command {
        return match cmd {
            CliCommand::Setup {
                force,
                yes,
                without_codex,
            } => Ok(CommandRequest::Setup(SetupRequest {
                force,
                yes,
                without_codex,
                path: args.path.to_string_lossy().into(),
            })),
        };
    }

    // Config source
    let config_source = if let Some(ref config_path) = args.config {
        ConfigSource::Explicit(config_path.to_string_lossy().into())
    } else if args.no_config {
        ConfigSource::None
    } else {
        ConfigSource::Auto
    };

    Ok(CommandRequest::List(ListRequest {
        path: args.path.to_string_lossy().into(),
        output_mode,
        depth: args.depth,
        sort: args.sort,
        config_source,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_list_request_default() {
        let args = CliArgs::try_parse_from(["lls"]).unwrap();
        let req = parse_request(args).unwrap();
        match req {
            CommandRequest::List(l) => {
                assert_eq!(l.path, ".");
                assert_eq!(l.output_mode, OutputMode::Json);
                assert_eq!(l.depth, 1);
                assert!(l.sort.is_none());
                assert!(matches!(l.config_source, ConfigSource::Auto));
            }
            _ => panic!("expected List"),
        }
    }

    #[test]
    fn test_setup_request() {
        let args = CliArgs::try_parse_from(["lls", "setup", "--without-codex"]).unwrap();
        let req = parse_request(args).unwrap();
        match req {
            CommandRequest::Setup(s) => {
                assert!(s.without_codex);
                assert!(!s.force);
            }
            _ => panic!("expected Setup"),
        }
    }

    #[test]
    fn test_no_config_source() {
        let args = CliArgs::try_parse_from(["lls", "--no-config"]).unwrap();
        let req = parse_request(args).unwrap();
        match req {
            CommandRequest::List(l) => {
                assert!(matches!(l.config_source, ConfigSource::None));
            }
            _ => panic!("expected List"),
        }
    }
}
