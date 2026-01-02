mod cli;
mod m3u;
mod output;
mod parser;
mod scanner;
mod types;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;
use m3u::{group_files, is_text_file, write_m3u};
use output::Output;
use scanner::scan_directory;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let output = Output::new(cli.quiet, cli.verbose);

    if let Err(e) = run(&cli, &output) {
        output.error(&format!("{:#}", e));
        std::process::exit(1);
    }
}

fn run(cli: &Cli, output: &Output) -> Result<()> {
    if cli.children {
        run_children_mode(cli, output)
    } else {
        run_normal_mode(cli, output)
    }
}

fn run_normal_mode(cli: &Cli, output: &Output) -> Result<()> {
    output.info(&format!("Scanning {}...", cli.target.display()));

    // Collect directories to process
    let dirs: Vec<_> = WalkDir::new(&cli.target)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.into_path())
        .collect();

    output.info(&format!("  Found {} directories to scan", dirs.len()));

    // Determine m3u destination
    let m3u_base = cli.destination.as_ref().unwrap_or(&cli.target);

    if let Some(dest) = &cli.destination {
        fs::create_dir_all(dest).context("Failed to create destination directory")?;
        check_and_clean_m3us(dest, output)?;
    }

    let pb = output.progress_bar(dirs.len() as u64);
    let mut total_m3us = 0;

    for dir in &dirs {
        pb.inc(1);

        let files = match scan_directory(dir) {
            Ok(f) => f,
            Err(e) => {
                output.warning(&format!("Could not scan {}: {}", dir.display(), e));
                continue;
            }
        };

        if files.is_empty() {
            continue;
        }

        output.verbose(&format!("Found {} media files in {}", files.len(), dir.display()));

        let m3u_dir = if cli.destination.is_some() {
            m3u_base.to_path_buf()
        } else {
            // Check and clean m3us in the source directory
            if let Err(e) = check_and_clean_m3us(dir, output) {
                output.warning(&format!("Skipping {}: {}", dir.display(), e));
                continue;
            }
            dir.to_path_buf()
        };

        let groups = group_files(files, cli.force);

        for group in groups {
            let m3u_path = m3u_dir.join(format!("{}.m3u", group.name));
            let relative_to = if cli.relative { Some(m3u_dir.as_path()) } else { None };

            write_m3u(&m3u_path, &group.files, relative_to)
                .with_context(|| format!("Failed to write {}", m3u_path.display()))?;

            output.verbose(&format!("Created {}", m3u_path.display()));
            total_m3us += 1;
        }
    }

    pb.finish_and_clear();
    output.success(&format!("Done: Created {} m3u files", total_m3us));

    Ok(())
}

fn run_children_mode(cli: &Cli, output: &Output) -> Result<()> {
    let dest = cli.destination.as_ref().expect("validated in cli");

    output.info(&format!("Scanning children of {}...", cli.target.display()));

    // Get top-level directories in target
    let children: Vec<_> = fs::read_dir(&cli.target)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter(|e| e.path() != *dest)
        .collect();

    if children.is_empty() {
        output.info("No subdirectories found in target");
        return Ok(());
    }

    output.info(&format!("  Found {} top-level directories", children.len()));
    let pb = output.progress_bar(children.len() as u64);
    let mut total_m3us = 0;

    for child in children {
        pb.inc(1);
        let child_path = child.path();
        let child_name = child.file_name();

        let m3u_dir = dest.join(&child_name);
        fs::create_dir_all(&m3u_dir)?;
        check_and_clean_m3us(&m3u_dir, output)?;

        // Recursively scan this child
        let dirs: Vec<_> = WalkDir::new(&child_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .map(|e| e.into_path())
            .collect();

        for dir in dirs {
            let files = match scan_directory(&dir) {
                Ok(f) => f,
                Err(e) => {
                    output.warning(&format!("Could not scan {}: {}", dir.display(), e));
                    continue;
                }
            };

            if files.is_empty() {
                continue;
            }

            let groups = group_files(files, cli.force);

            for group in groups {
                let m3u_path = m3u_dir.join(format!("{}.m3u", group.name));
                let relative_to = if cli.relative { Some(m3u_dir.as_path()) } else { None };

                write_m3u(&m3u_path, &group.files, relative_to)?;
                output.verbose(&format!("Created {}", m3u_path.display()));
                total_m3us += 1;
            }
        }

        // Remove empty directories
        if fs::read_dir(&m3u_dir)?.next().is_none() {
            fs::remove_dir(&m3u_dir)?;
        }
    }

    pb.finish_and_clear();
    output.success(&format!("Done: Created {} m3u files", total_m3us));

    Ok(())
}

fn check_and_clean_m3us(dir: &Path, output: &Output) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map(|e| e.to_ascii_lowercase()) == Some("m3u".into()) {
            if !is_text_file(&path)? {
                anyhow::bail!(
                    "{} may not be a text file, refusing to delete",
                    path.display()
                );
            }
            fs::remove_file(&path)?;
            output.verbose(&format!("Removed old {}", path.display()));
        }
    }
    Ok(())
}
