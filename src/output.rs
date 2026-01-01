// src/output.rs
use console::{style, Term};
use indicatif::{ProgressBar, ProgressStyle};

pub struct Output {
    quiet: bool,
    verbose: bool,
    term: Term,
}

impl Output {
    pub fn new(quiet: bool, verbose: bool) -> Self {
        Self {
            quiet,
            verbose,
            term: Term::stderr(),
        }
    }

    pub fn info(&self, msg: &str) {
        if !self.quiet {
            let _ = self.term.write_line(msg);
        }
    }

    pub fn verbose(&self, msg: &str) {
        if self.verbose && !self.quiet {
            let _ = self.term.write_line(&format!("  {}", style(msg).dim()));
        }
    }

    pub fn warning(&self, msg: &str) {
        let _ = self.term.write_line(&format!("{}: {}", style("Warning").yellow(), msg));
    }

    pub fn error(&self, msg: &str) {
        let _ = self.term.write_line(&format!("{}: {}", style("Error").red(), msg));
    }

    pub fn success(&self, msg: &str) {
        if !self.quiet {
            let _ = self.term.write_line(&format!("{}", style(msg).green()));
        }
    }

    pub fn progress_bar(&self, len: u64) -> ProgressBar {
        if self.quiet {
            return ProgressBar::hidden();
        }

        let pb = ProgressBar::new(len);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("  [{bar:40.cyan/blue}] {pos}/{len} directories")
                .unwrap()
                .progress_chars("█▓░"),
        );
        pb
    }
}
