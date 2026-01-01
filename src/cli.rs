use clap::Parser;
use std::path::PathBuf;

/// Generate m3u playlists for multi-disc ROM collections
#[derive(Parser, Debug)]
#[command(name = "m3u-emu", version, about)]
pub struct Cli {
    /// Directory to search for ROM media files
    #[arg(required = true)]
    pub target: PathBuf,

    /// Where to write m3u files (default: alongside ROMs)
    #[arg()]
    pub destination: Option<PathBuf>,

    /// Use relative paths in m3u files (default: absolute)
    #[arg(short, long)]
    pub relative: bool,

    /// Create subdirectories in DESTINATION mirroring TARGET's top-level folders
    #[arg(short, long)]
    pub children: bool,

    /// Use disc-style grouping for floppy formats too
    #[arg(short, long)]
    pub force: bool,

    /// Suppress progress output, only show errors
    #[arg(short, long, conflicts_with = "verbose")]
    pub quiet: bool,

    /// Show detailed information about each file processed
    #[arg(short, long, conflicts_with = "quiet")]
    pub verbose: bool,
}

impl Cli {
    pub fn validate(&self) -> Result<(), String> {
        if self.children && self.destination.is_none() {
            return Err("the --children flag requires a DESTINATION".to_string());
        }
        if !self.target.exists() {
            return Err(format!(
                "target directory does not exist: {}",
                self.target.display()
            ));
        }
        if !self.target.is_dir() {
            return Err(format!(
                "target is not a directory: {}",
                self.target.display()
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_basic() {
        let cli = Cli::parse_from(["m3u-emu", "/some/path"]);
        assert_eq!(cli.target, PathBuf::from("/some/path"));
        assert!(cli.destination.is_none());
        assert!(!cli.relative);
        assert!(!cli.children);
        assert!(!cli.force);
    }

    #[test]
    fn test_cli_parse_all_flags() {
        let cli = Cli::parse_from([
            "m3u-emu", "-r", "-c", "-f", "-q", "/target", "/dest"
        ]);
        assert!(cli.relative);
        assert!(cli.children);
        assert!(cli.force);
        assert!(cli.quiet);
        assert_eq!(cli.destination, Some(PathBuf::from("/dest")));
    }

    #[test]
    fn test_cli_parse_long_flags() {
        let cli = Cli::parse_from([
            "m3u-emu", "--relative", "--verbose", "/target"
        ]);
        assert!(cli.relative);
        assert!(cli.verbose);
    }
}
