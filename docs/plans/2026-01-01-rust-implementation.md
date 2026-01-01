# m3u-emu Rust Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Port the bash m3u playlist generator to a cross-platform Rust CLI tool.

**Architecture:** CLI parses args via clap, scanner walks directories finding media files, parser extracts game names and disc numbers from filenames, m3u module groups files and writes playlists. Progress shown via indicatif.

**Tech Stack:** Rust, clap (CLI), walkdir (traversal), regex (patterns), infer (file type detection), indicatif + console (progress/color), anyhow (errors)

---

### Task 1: Set Up Dependencies

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add dependencies to Cargo.toml**

```toml
[package]
name = "m3u-emu"
version = "0.1.0"
edition = "2021"
description = "Generate m3u playlists for multi-disc ROM collections"

[dependencies]
clap = { version = "4", features = ["derive"] }
walkdir = "2"
regex = "1"
infer = "0.16"
indicatif = "0.17"
console = "0.15"
anyhow = "1"
once_cell = "1"

[dev-dependencies]
tempfile = "3"
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles successfully (downloads dependencies)

**Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "chore: initialize Rust project with dependencies"
```

---

### Task 2: Define Core Types

**Files:**
- Create: `src/types.rs`
- Modify: `src/main.rs`

**Step 1: Write tests for MediaType**

```rust
// src/types.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_from_extension_floppy() {
        assert_eq!(MediaType::from_extension("adf"), Some(MediaType::Floppy));
        assert_eq!(MediaType::from_extension("ADF"), Some(MediaType::Floppy));
        assert_eq!(MediaType::from_extension("d64"), Some(MediaType::Floppy));
    }

    #[test]
    fn test_media_type_from_extension_disc_index() {
        assert_eq!(MediaType::from_extension("cue"), Some(MediaType::DiscIndex));
        assert_eq!(MediaType::from_extension("CUE"), Some(MediaType::DiscIndex));
        assert_eq!(MediaType::from_extension("gdi"), Some(MediaType::DiscIndex));
    }

    #[test]
    fn test_media_type_from_extension_disc_image() {
        assert_eq!(MediaType::from_extension("iso"), Some(MediaType::DiscImage));
        assert_eq!(MediaType::from_extension("chd"), Some(MediaType::DiscImage));
    }

    #[test]
    fn test_media_type_from_extension_unknown() {
        assert_eq!(MediaType::from_extension("txt"), None);
        assert_eq!(MediaType::from_extension("exe"), None);
    }

    #[test]
    fn test_media_type_is_floppy() {
        assert!(MediaType::Floppy.is_floppy());
        assert!(!MediaType::DiscIndex.is_floppy());
        assert!(!MediaType::DiscImage.is_floppy());
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib types`
Expected: FAIL - module not found

**Step 3: Implement MediaType**

```rust
// src/types.rs
use std::path::{Path, PathBuf};

/// Type of media file detected from extension
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaType {
    /// Floppy disk formats (adf, d64, etc.)
    Floppy,
    /// Disc index formats that reference images (cue, toc, ccd, gdi)
    DiscIndex,
    /// Disc image formats (iso, chd, etc.)
    DiscImage,
}

const FLOPPY_EXTENSIONS: &[&str] = &[
    "ipf", "adf", "adz", "dms", "dim", "d64", "d71", "d81", "d88",
    "dsk", "ima", "fdi", "qd", "fds", "tap", "tzx", "cas",
];

const DISC_INDEX_EXTENSIONS: &[&str] = &["cue", "toc", "ccd", "gdi"];

const DISC_IMAGE_EXTENSIONS: &[&str] = &["mds", "cdi", "img", "iso", "chd", "rvz"];

impl MediaType {
    /// Detect media type from file extension (case-insensitive)
    pub fn from_extension(ext: &str) -> Option<MediaType> {
        let ext_lower = ext.to_lowercase();
        if FLOPPY_EXTENSIONS.contains(&ext_lower.as_str()) {
            Some(MediaType::Floppy)
        } else if DISC_INDEX_EXTENSIONS.contains(&ext_lower.as_str()) {
            Some(MediaType::DiscIndex)
        } else if DISC_IMAGE_EXTENSIONS.contains(&ext_lower.as_str()) {
            Some(MediaType::DiscImage)
        } else {
            None
        }
    }

    /// Check if this is a floppy format
    pub fn is_floppy(&self) -> bool {
        matches!(self, MediaType::Floppy)
    }
}

/// A media file with parsed metadata
#[derive(Debug, Clone)]
pub struct MediaFile {
    /// Full path to the file
    pub path: PathBuf,
    /// Just the filename (no directory)
    pub filename: String,
    /// Name with disc/side markers removed (for grouping)
    pub base_name: String,
    /// Disc number, with side as decimal (e.g., 2.1 for Disc 2 Side 1)
    pub disc_number: f32,
    /// The detected media type
    pub media_type: MediaType,
}

impl MediaFile {
    pub fn is_floppy(&self) -> bool {
        self.media_type.is_floppy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_type_from_extension_floppy() {
        assert_eq!(MediaType::from_extension("adf"), Some(MediaType::Floppy));
        assert_eq!(MediaType::from_extension("ADF"), Some(MediaType::Floppy));
        assert_eq!(MediaType::from_extension("d64"), Some(MediaType::Floppy));
    }

    #[test]
    fn test_media_type_from_extension_disc_index() {
        assert_eq!(MediaType::from_extension("cue"), Some(MediaType::DiscIndex));
        assert_eq!(MediaType::from_extension("CUE"), Some(MediaType::DiscIndex));
        assert_eq!(MediaType::from_extension("gdi"), Some(MediaType::DiscIndex));
    }

    #[test]
    fn test_media_type_from_extension_disc_image() {
        assert_eq!(MediaType::from_extension("iso"), Some(MediaType::DiscImage));
        assert_eq!(MediaType::from_extension("chd"), Some(MediaType::DiscImage));
    }

    #[test]
    fn test_media_type_from_extension_unknown() {
        assert_eq!(MediaType::from_extension("txt"), None);
        assert_eq!(MediaType::from_extension("exe"), None);
    }

    #[test]
    fn test_media_type_is_floppy() {
        assert!(MediaType::Floppy.is_floppy());
        assert!(!MediaType::DiscIndex.is_floppy());
        assert!(!MediaType::DiscImage.is_floppy());
    }
}
```

**Step 4: Update main.rs to include module**

```rust
// src/main.rs
mod types;

fn main() {
    println!("m3u-emu");
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test --lib types`
Expected: All 5 tests pass

**Step 6: Commit**

```bash
git add src/types.rs src/main.rs
git commit -m "feat: add MediaType and MediaFile types"
```

---

### Task 3: Implement CLI Module

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs`

**Step 1: Write CLI struct with clap derive**

```rust
// src/cli.rs
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
    #[arg(short, long)]
    pub quiet: bool,

    /// Show detailed information about each file processed
    #[arg(short, long)]
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
```

**Step 2: Run tests to verify they pass**

Run: `cargo test --lib cli`
Expected: All 3 tests pass

**Step 3: Update main.rs to use CLI**

```rust
// src/main.rs
mod cli;
mod types;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    println!("Target: {:?}", cli.target);
}
```

**Step 4: Verify CLI works**

Run: `cargo run -- --help`
Expected: Shows help text with all options

**Step 5: Commit**

```bash
git add src/cli.rs src/main.rs
git commit -m "feat: add CLI argument parsing with clap"
```

---

### Task 4: Implement Word-to-Number Conversion

**Files:**
- Create: `src/parser.rs`
- Modify: `src/main.rs`

**Step 1: Write tests for word/letter to number conversion**

```rust
// src/parser.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_to_number_numerals() {
        assert_eq!(word_to_number("one"), Some(1));
        assert_eq!(word_to_number("two"), Some(2));
        assert_eq!(word_to_number("ten"), Some(10));
        assert_eq!(word_to_number("twenty-three"), Some(23));
        assert_eq!(word_to_number("thirty-one"), Some(31));
    }

    #[test]
    fn test_word_to_number_case_insensitive() {
        assert_eq!(word_to_number("ONE"), Some(1));
        assert_eq!(word_to_number("Twenty-Three"), Some(23));
    }

    #[test]
    fn test_word_to_number_alpha() {
        assert_eq!(word_to_number("A"), Some(1));
        assert_eq!(word_to_number("B"), Some(2));
        assert_eq!(word_to_number("Z"), Some(26));
    }

    #[test]
    fn test_word_to_number_alpha_case_sensitive() {
        // Lowercase letters should NOT match (only uppercase A-Z)
        assert_eq!(word_to_number("a"), None);
        assert_eq!(word_to_number("b"), None);
    }

    #[test]
    fn test_word_to_number_special() {
        assert_eq!(word_to_number("boot"), Some(0));
        assert_eq!(word_to_number("save"), Some(99));
        assert_eq!(word_to_number("BOOT"), Some(0));
    }

    #[test]
    fn test_word_to_number_invalid() {
        assert_eq!(word_to_number("hello"), None);
        assert_eq!(word_to_number(""), None);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib parser`
Expected: FAIL - module not found

**Step 3: Implement word_to_number**

```rust
// src/parser.rs
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Convert English word numerals and single letters to numbers
pub fn word_to_number(word: &str) -> Option<u32> {
    static NUMERALS: Lazy<HashMap<&'static str, u32>> = Lazy::new(|| {
        let mut m = HashMap::new();
        m.insert("zero", 0);
        m.insert("one", 1);
        m.insert("two", 2);
        m.insert("three", 3);
        m.insert("four", 4);
        m.insert("five", 5);
        m.insert("six", 6);
        m.insert("seven", 7);
        m.insert("eight", 8);
        m.insert("nine", 9);
        m.insert("ten", 10);
        m.insert("eleven", 11);
        m.insert("twelve", 12);
        m.insert("thirteen", 13);
        m.insert("fourteen", 14);
        m.insert("fifteen", 15);
        m.insert("sixteen", 16);
        m.insert("seventeen", 17);
        m.insert("eighteen", 18);
        m.insert("nineteen", 19);
        m.insert("twenty", 20);
        m.insert("twenty-one", 21);
        m.insert("twenty-two", 22);
        m.insert("twenty-three", 23);
        m.insert("twenty-four", 24);
        m.insert("twenty-five", 25);
        m.insert("twenty-six", 26);
        m.insert("twenty-seven", 27);
        m.insert("twenty-eight", 28);
        m.insert("twenty-nine", 29);
        m.insert("thirty", 30);
        m.insert("thirty-one", 31);
        m.insert("boot", 0);
        m.insert("save", 99);
        m
    });

    let lower = word.to_lowercase();

    // Check numerals (case-insensitive)
    if let Some(&n) = NUMERALS.get(lower.as_str()) {
        return Some(n);
    }

    // Check single uppercase letters A-Z (case-sensitive!)
    if word.len() == 1 {
        let ch = word.chars().next().unwrap();
        if ch.is_ascii_uppercase() {
            return Some((ch as u32) - ('A' as u32) + 1);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_to_number_numerals() {
        assert_eq!(word_to_number("one"), Some(1));
        assert_eq!(word_to_number("two"), Some(2));
        assert_eq!(word_to_number("ten"), Some(10));
        assert_eq!(word_to_number("twenty-three"), Some(23));
        assert_eq!(word_to_number("thirty-one"), Some(31));
    }

    #[test]
    fn test_word_to_number_case_insensitive() {
        assert_eq!(word_to_number("ONE"), Some(1));
        assert_eq!(word_to_number("Twenty-Three"), Some(23));
    }

    #[test]
    fn test_word_to_number_alpha() {
        assert_eq!(word_to_number("A"), Some(1));
        assert_eq!(word_to_number("B"), Some(2));
        assert_eq!(word_to_number("Z"), Some(26));
    }

    #[test]
    fn test_word_to_number_alpha_case_sensitive() {
        assert_eq!(word_to_number("a"), None);
        assert_eq!(word_to_number("b"), None);
    }

    #[test]
    fn test_word_to_number_special() {
        assert_eq!(word_to_number("boot"), Some(0));
        assert_eq!(word_to_number("save"), Some(99));
        assert_eq!(word_to_number("BOOT"), Some(0));
    }

    #[test]
    fn test_word_to_number_invalid() {
        assert_eq!(word_to_number("hello"), None);
        assert_eq!(word_to_number(""), None);
    }
}
```

**Step 4: Update main.rs**

```rust
// src/main.rs
mod cli;
mod parser;
mod types;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    println!("Target: {:?}", cli.target);
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test --lib parser`
Expected: All 6 tests pass

**Step 6: Commit**

```bash
git add src/parser.rs src/main.rs
git commit -m "feat: add word-to-number conversion for disc identifiers"
```

---

### Task 5: Implement Number Extraction

**Files:**
- Modify: `src/parser.rs`

**Step 1: Add tests for extract_number**

Add to the test module in `src/parser.rs`:

```rust
    #[test]
    fn test_extract_number_digits() {
        assert_eq!(extract_number("2"), Some(2));
        assert_eq!(extract_number("12"), Some(12));
        assert_eq!(extract_number("007"), Some(7)); // Leading zeros removed
    }

    #[test]
    fn test_extract_number_with_text() {
        assert_eq!(extract_number("Disc 2"), Some(2));
        assert_eq!(extract_number("CD 12"), Some(12));
    }

    #[test]
    fn test_extract_number_word() {
        assert_eq!(extract_number("Disc Two"), Some(2));
        assert_eq!(extract_number("CD Twenty-Three"), Some(23));
    }

    #[test]
    fn test_extract_number_letter() {
        assert_eq!(extract_number("Disk A"), Some(1));
        assert_eq!(extract_number("Floppy B"), Some(2));
    }

    #[test]
    fn test_extract_number_single_word() {
        assert_eq!(extract_number("boot"), Some(0));
        assert_eq!(extract_number("A"), Some(1));
    }

    #[test]
    fn test_extract_number_none() {
        assert_eq!(extract_number(""), None);
        assert_eq!(extract_number("hello world"), None);
    }
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib parser::tests::test_extract`
Expected: FAIL - function not found

**Step 3: Implement extract_number**

Add to `src/parser.rs` after the `word_to_number` function:

```rust
static NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d+").unwrap());

/// Extract a number from a string containing digits, words, or letters
pub fn extract_number(s: &str) -> Option<u32> {
    // First try to find a numeric digit sequence
    if let Some(m) = NUMBER_REGEX.find(s) {
        if let Ok(n) = m.as_str().parse::<u32>() {
            return Some(n);
        }
    }

    // Split into words and try to convert
    let words: Vec<&str> = s.split_whitespace().collect();

    if words.is_empty() {
        return None;
    }

    if words.len() == 1 {
        return word_to_number(words[0]);
    }

    // Try the second word (e.g., "Disc Two" -> "Two")
    if words.len() >= 2 {
        if let Some(n) = word_to_number(words[1]) {
            return Some(n);
        }
    }

    None
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib parser`
Expected: All 12 tests pass

**Step 5: Commit**

```bash
git add src/parser.rs
git commit -m "feat: add extract_number for parsing disc identifiers"
```

---

### Task 6: Implement Filename Parsing

**Files:**
- Modify: `src/parser.rs`
- Modify: `src/types.rs`

**Step 1: Add tests for parse_filename**

Add to the test module in `src/parser.rs`:

```rust
    #[test]
    fn test_parse_filename_disc() {
        let result = parse_filename("Final Fantasy VII (Disc 2).cue");
        assert_eq!(result.base_name, "Final Fantasy VII");
        assert_eq!(result.disc_number, 2.0);
    }

    #[test]
    fn test_parse_filename_cd() {
        let result = parse_filename("Game (CD 1).iso");
        assert_eq!(result.base_name, "Game");
        assert_eq!(result.disc_number, 1.0);
    }

    #[test]
    fn test_parse_filename_floppy() {
        let result = parse_filename("Monkey Island (Disk A).adf");
        assert_eq!(result.base_name, "Monkey Island");
        assert_eq!(result.disc_number, 1.0);
    }

    #[test]
    fn test_parse_filename_with_side() {
        let result = parse_filename("Game (Disk 1) (Side A).adf");
        assert_eq!(result.base_name, "Game");
        assert!((result.disc_number - 1.1).abs() < 0.01);
    }

    #[test]
    fn test_parse_filename_no_disc() {
        let result = parse_filename("Single Game.iso");
        assert_eq!(result.base_name, "Single Game");
        assert_eq!(result.disc_number, 1.0);
    }

    #[test]
    fn test_parse_filename_word_number() {
        let result = parse_filename("Game (Disc Two).cue");
        assert_eq!(result.base_name, "Game");
        assert_eq!(result.disc_number, 2.0);
    }

    #[test]
    fn test_parse_filename_boot_save() {
        let boot = parse_filename("Game (Boot).adf");
        assert_eq!(boot.disc_number, 0.0);

        let save = parse_filename("Game (Save).adf");
        assert_eq!(save.disc_number, 99.0);
    }

    #[test]
    fn test_parse_filename_preserves_other_parens() {
        let result = parse_filename("Game (USA) (Disc 1).cue");
        assert_eq!(result.base_name, "Game (USA)");
        assert_eq!(result.disc_number, 1.0);
    }
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib parser::tests::test_parse_filename`
Expected: FAIL - function not found

**Step 3: Implement parse_filename**

Add to `src/parser.rs`:

```rust
/// Result of parsing a filename
#[derive(Debug, Clone)]
pub struct ParsedFilename {
    pub base_name: String,
    pub disc_number: f32,
}

static ORDER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\(\s*(floppy\s|diskette\s|disk\s|cd\s|disc\s|boot|save)[^)(]*\)").unwrap()
});

static SIDE_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\(\s*side\s[^)(]*\)").unwrap()
});

/// Parse a filename to extract base name and disc number
pub fn parse_filename(filename: &str) -> ParsedFilename {
    // Remove extension
    let name = match filename.rfind('.') {
        Some(i) => &filename[..i],
        None => filename,
    };

    let mut base_name = name.to_string();
    let mut disc_number: f32 = 1.0;

    // Extract disc/cd/floppy identifier
    if let Some(m) = ORDER_REGEX.find(name) {
        let matched = m.as_str();
        // Extract just the content inside parentheses
        let inner = &matched[1..matched.len() - 1];
        if let Some(n) = extract_number(inner) {
            disc_number = n as f32;
        }
        // Remove the match from base_name
        base_name = ORDER_REGEX.replace_all(&base_name, "").to_string();
    }

    // Extract side identifier
    if let Some(m) = SIDE_REGEX.find(name) {
        let matched = m.as_str();
        let inner = &matched[1..matched.len() - 1];
        if let Some(n) = extract_number(inner) {
            disc_number += n as f32 * 0.1;
        }
        base_name = SIDE_REGEX.replace_all(&base_name, "").to_string();
    }

    // Clean up whitespace
    let base_name = base_name
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    ParsedFilename {
        base_name,
        disc_number,
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib parser`
Expected: All 20 tests pass

**Step 5: Commit**

```bash
git add src/parser.rs
git commit -m "feat: add filename parsing for disc/side extraction"
```

---

### Task 7: Implement M3U Validation

**Files:**
- Create: `src/m3u.rs`
- Modify: `src/main.rs`

**Step 1: Write tests for is_text_file**

```rust
// src/m3u.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_is_text_file_valid_m3u() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "/path/to/game.cue").unwrap();
        writeln!(f, "/path/to/game2.cue").unwrap();
        assert!(is_text_file(f.path()).unwrap());
    }

    #[test]
    fn test_is_text_file_binary() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(&[0x00, 0x01, 0x02, 0xFF, 0xFE]).unwrap();
        assert!(!is_text_file(f.path()).unwrap());
    }

    #[test]
    fn test_is_text_file_empty() {
        let f = NamedTempFile::new().unwrap();
        assert!(is_text_file(f.path()).unwrap());
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib m3u`
Expected: FAIL - module not found

**Step 3: Implement is_text_file**

```rust
// src/m3u.rs
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Check if a file appears to be a text file (not binary)
pub fn is_text_file(path: &Path) -> Result<bool> {
    let bytes = fs::read(path)?;

    if bytes.is_empty() {
        return Ok(true);
    }

    // Use infer to check if it's a known binary format
    if let Some(kind) = infer::get(&bytes) {
        // If infer recognizes it as something, it's probably not plain text
        // Exception: we don't care about text-based formats infer might detect
        let mime = kind.mime_type();
        if !mime.starts_with("text/") {
            return Ok(false);
        }
    }

    // Check for null bytes and high ratio of non-printable characters
    let non_text_count = bytes
        .iter()
        .filter(|&&b| b == 0 || (b < 32 && b != 9 && b != 10 && b != 13))
        .count();

    // If more than 10% non-text bytes, probably binary
    Ok(non_text_count < bytes.len() / 10)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_is_text_file_valid_m3u() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "/path/to/game.cue").unwrap();
        writeln!(f, "/path/to/game2.cue").unwrap();
        assert!(is_text_file(f.path()).unwrap());
    }

    #[test]
    fn test_is_text_file_binary() {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(&[0x00, 0x01, 0x02, 0xFF, 0xFE]).unwrap();
        assert!(!is_text_file(f.path()).unwrap());
    }

    #[test]
    fn test_is_text_file_empty() {
        let f = NamedTempFile::new().unwrap();
        assert!(is_text_file(f.path()).unwrap());
    }
}
```

**Step 4: Update main.rs**

```rust
// src/main.rs
mod cli;
mod m3u;
mod parser;
mod types;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    println!("Target: {:?}", cli.target);
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test --lib m3u`
Expected: All 3 tests pass

**Step 6: Commit**

```bash
git add src/m3u.rs src/main.rs
git commit -m "feat: add m3u text file validation"
```

---

### Task 8: Implement Directory Scanning

**Files:**
- Create: `src/scanner.rs`
- Modify: `src/main.rs`

**Step 1: Write tests for scan_directory**

```rust
// src/scanner.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn create_test_files(dir: &Path, names: &[&str]) {
        for name in names {
            File::create(dir.join(name)).unwrap();
        }
    }

    #[test]
    fn test_scan_directory_finds_cue_files() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path(), &["game.cue", "game.bin", "readme.txt"]);

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].filename.ends_with(".cue"));
    }

    #[test]
    fn test_scan_directory_finds_floppy_files() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path(), &["game.adf", "game2.d64"]);

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scan_directory_prefers_index_over_image() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path(), &["game.cue", "game.iso", "game.bin"]);

        let files = scan_directory(dir.path()).unwrap();
        // Should only return .cue since it's an index format
        assert_eq!(files.len(), 1);
        assert!(files[0].filename.ends_with(".cue"));
    }

    #[test]
    fn test_scan_directory_empty() {
        let dir = TempDir::new().unwrap();
        let files = scan_directory(dir.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_scan_directory_case_insensitive() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path(), &["game.CUE", "game.ISO"]);

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 1); // .CUE preferred over .ISO
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib scanner`
Expected: FAIL - module not found

**Step 3: Implement scan_directory**

```rust
// src/scanner.rs
use crate::parser::parse_filename;
use crate::types::{MediaFile, MediaType};
use anyhow::Result;
use std::fs;
use std::path::Path;

/// Scan a single directory for media files
pub fn scan_directory(dir: &Path) -> Result<Vec<MediaFile>> {
    let mut floppy_files = Vec::new();
    let mut disc_index_files = Vec::new();
    let mut disc_image_files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_string(),
            None => continue,
        };

        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e,
            None => continue,
        };

        let media_type = match MediaType::from_extension(ext) {
            Some(t) => t,
            None => continue,
        };

        let parsed = parse_filename(&filename);
        let media_file = MediaFile {
            path,
            filename,
            base_name: parsed.base_name,
            disc_number: parsed.disc_number,
            media_type,
        };

        match media_type {
            MediaType::Floppy => floppy_files.push(media_file),
            MediaType::DiscIndex => disc_index_files.push(media_file),
            MediaType::DiscImage => disc_image_files.push(media_file),
        }
    }

    // Priority: floppy > disc index > disc image
    // If we have floppy files, return those
    // If we have disc index files, return those (not images)
    // Otherwise return disc images
    if !floppy_files.is_empty() {
        Ok(floppy_files)
    } else if !disc_index_files.is_empty() {
        Ok(disc_index_files)
    } else {
        Ok(disc_image_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn create_test_files(dir: &Path, names: &[&str]) {
        for name in names {
            File::create(dir.join(name)).unwrap();
        }
    }

    #[test]
    fn test_scan_directory_finds_cue_files() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path(), &["game.cue", "game.bin", "readme.txt"]);

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].filename.ends_with(".cue"));
    }

    #[test]
    fn test_scan_directory_finds_floppy_files() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path(), &["game.adf", "game2.d64"]);

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_scan_directory_prefers_index_over_image() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path(), &["game.cue", "game.iso", "game.bin"]);

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].filename.ends_with(".cue"));
    }

    #[test]
    fn test_scan_directory_empty() {
        let dir = TempDir::new().unwrap();
        let files = scan_directory(dir.path()).unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_scan_directory_case_insensitive() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path(), &["game.CUE", "game.ISO"]);

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }
}
```

**Step 4: Update main.rs**

```rust
// src/main.rs
mod cli;
mod m3u;
mod parser;
mod scanner;
mod types;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    println!("Target: {:?}", cli.target);
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test --lib scanner`
Expected: All 5 tests pass

**Step 6: Commit**

```bash
git add src/scanner.rs src/main.rs
git commit -m "feat: add directory scanning for media files"
```

---

### Task 9: Implement File Grouping

**Files:**
- Modify: `src/m3u.rs`

**Step 1: Add tests for group_files**

Add to tests in `src/m3u.rs`:

```rust
    #[test]
    fn test_group_files_disc_mode() {
        let files = vec![
            make_media_file("FF7 (Disc 1).cue", "FF7", 1.0, false),
            make_media_file("FF7 (Disc 2).cue", "FF7", 2.0, false),
            make_media_file("FF8 (Disc 1).cue", "FF8", 1.0, false),
        ];

        let groups = group_files(files, false);
        assert_eq!(groups.len(), 2);

        let ff7 = groups.iter().find(|g| g.name == "FF7").unwrap();
        assert_eq!(ff7.files.len(), 2);

        let ff8 = groups.iter().find(|g| g.name == "FF8").unwrap();
        assert_eq!(ff8.files.len(), 1);
    }

    #[test]
    fn test_group_files_floppy_mode() {
        let files = vec![
            make_media_file("Game (Disk A).adf", "Game", 1.0, true),
            make_media_file("Game (Disk B).adf", "Game", 2.0, true),
            make_media_file("Other (Boot).adf", "Other", 0.0, true),
        ];

        // In floppy mode without force, all files go into one group
        let groups = group_files(files, false);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].files.len(), 3);
    }

    #[test]
    fn test_group_files_floppy_with_force() {
        let files = vec![
            make_media_file("Game (Disk A).adf", "Game", 1.0, true),
            make_media_file("Game (Disk B).adf", "Game", 2.0, true),
            make_media_file("Other (Boot).adf", "Other", 0.0, true),
        ];

        // With force, group by name like disc mode
        let groups = group_files(files, true);
        assert_eq!(groups.len(), 2);
    }

    #[test]
    fn test_group_files_sorted() {
        let files = vec![
            make_media_file("Game (Disc 3).cue", "Game", 3.0, false),
            make_media_file("Game (Disc 1).cue", "Game", 1.0, false),
            make_media_file("Game (Disc 2).cue", "Game", 2.0, false),
        ];

        let groups = group_files(files, false);
        let game = &groups[0];

        assert_eq!(game.files[0].disc_number, 1.0);
        assert_eq!(game.files[1].disc_number, 2.0);
        assert_eq!(game.files[2].disc_number, 3.0);
    }

    fn make_media_file(filename: &str, base_name: &str, disc: f32, floppy: bool) -> MediaFile {
        MediaFile {
            path: PathBuf::from(filename),
            filename: filename.to_string(),
            base_name: base_name.to_string(),
            disc_number: disc,
            media_type: if floppy { MediaType::Floppy } else { MediaType::DiscIndex },
        }
    }
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib m3u::tests::test_group`
Expected: FAIL - function not found

**Step 3: Implement group_files**

Add to `src/m3u.rs`:

```rust
use crate::types::{MediaFile, MediaType};
use std::path::PathBuf;

/// A group of files that will become one m3u
#[derive(Debug)]
pub struct GameSet {
    pub name: String,
    pub files: Vec<MediaFile>,
}

/// Group media files into game sets for m3u creation
pub fn group_files(mut files: Vec<MediaFile>, force_disc_mode: bool) -> Vec<GameSet> {
    if files.is_empty() {
        return Vec::new();
    }

    // Check if any file is floppy format
    let has_floppy = files.iter().any(|f| f.is_floppy());
    let use_floppy_mode = has_floppy && !force_disc_mode;

    // Sort files by disc number for consistent ordering
    files.sort_by(|a, b| {
        a.disc_number
            .partial_cmp(&b.disc_number)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if use_floppy_mode {
        // Floppy mode: all files in one group, use first file's base_name
        let name = files.first().map(|f| f.base_name.clone()).unwrap_or_default();
        return vec![GameSet { name, files }];
    }

    // Disc mode: group by base_name
    let mut groups: Vec<GameSet> = Vec::new();

    for file in files {
        if let Some(group) = groups.iter_mut().find(|g| g.name == file.base_name) {
            group.files.push(file);
        } else {
            groups.push(GameSet {
                name: file.base_name.clone(),
                files: vec![file],
            });
        }
    }

    // Sort files within each group by disc number
    for group in &mut groups {
        group.files.sort_by(|a, b| {
            a.disc_number
                .partial_cmp(&b.disc_number)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    groups
}
```

Also update the imports at the top and add the test helper:

```rust
// At top of src/m3u.rs
use crate::types::{MediaFile, MediaType};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --lib m3u`
Expected: All 7 tests pass

**Step 5: Commit**

```bash
git add src/m3u.rs
git commit -m "feat: add file grouping for m3u generation"
```

---

### Task 10: Implement M3U Writing

**Files:**
- Modify: `src/m3u.rs`

**Step 1: Add tests for write_m3u**

Add to tests in `src/m3u.rs`:

```rust
    #[test]
    fn test_write_m3u_absolute() {
        let dir = TempDir::new().unwrap();
        let game_dir = dir.path().join("games");
        fs::create_dir(&game_dir).unwrap();

        let files = vec![
            MediaFile {
                path: game_dir.join("Game (Disc 1).cue"),
                filename: "Game (Disc 1).cue".to_string(),
                base_name: "Game".to_string(),
                disc_number: 1.0,
                media_type: MediaType::DiscIndex,
            },
        ];

        let m3u_path = dir.path().join("Game.m3u");
        write_m3u(&m3u_path, &files, None).unwrap();

        let content = fs::read_to_string(&m3u_path).unwrap();
        assert!(content.contains(&game_dir.join("Game (Disc 1).cue").to_string_lossy().to_string()));
    }

    #[test]
    fn test_write_m3u_relative() {
        let dir = TempDir::new().unwrap();
        let game_dir = dir.path().join("games");
        fs::create_dir(&game_dir).unwrap();

        let files = vec![
            MediaFile {
                path: game_dir.join("Game (Disc 1).cue"),
                filename: "Game (Disc 1).cue".to_string(),
                base_name: "Game".to_string(),
                disc_number: 1.0,
                media_type: MediaType::DiscIndex,
            },
        ];

        let m3u_path = dir.path().join("Game.m3u");
        write_m3u(&m3u_path, &files, Some(dir.path())).unwrap();

        let content = fs::read_to_string(&m3u_path).unwrap();
        assert!(content.contains("games/Game (Disc 1).cue") || content.contains("games\\Game (Disc 1).cue"));
    }
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --lib m3u::tests::test_write`
Expected: FAIL - function not found

**Step 3: Implement write_m3u**

Add to `src/m3u.rs`:

```rust
use std::io::Write;

/// Write an m3u file with paths to the given media files
pub fn write_m3u(
    m3u_path: &Path,
    files: &[MediaFile],
    relative_to: Option<&Path>,
) -> Result<()> {
    let mut f = fs::File::create(m3u_path)?;

    for media_file in files {
        let path_str = match relative_to {
            Some(base) => {
                // Calculate relative path
                pathdiff::diff_paths(&media_file.path, base)
                    .unwrap_or_else(|| media_file.path.clone())
                    .to_string_lossy()
                    .to_string()
            }
            None => media_file.path.to_string_lossy().to_string(),
        };
        writeln!(f, "{}", path_str)?;
    }

    Ok(())
}
```

**Step 4: Add pathdiff dependency**

Update `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
walkdir = "2"
regex = "1"
infer = "0.16"
indicatif = "0.17"
console = "0.15"
anyhow = "1"
once_cell = "1"
pathdiff = "0.2"

[dev-dependencies]
tempfile = "3"
```

**Step 5: Run tests to verify they pass**

Run: `cargo test --lib m3u`
Expected: All 9 tests pass

**Step 6: Commit**

```bash
git add src/m3u.rs Cargo.toml Cargo.lock
git commit -m "feat: add m3u file writing with relative path support"
```

---

### Task 11: Implement Progress Output

**Files:**
- Create: `src/output.rs`
- Modify: `src/main.rs`

**Step 1: Create output module**

```rust
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
```

**Step 2: Update main.rs**

```rust
// src/main.rs
mod cli;
mod m3u;
mod output;
mod parser;
mod scanner;
mod types;

use clap::Parser;
use cli::Cli;
use output::Output;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.validate() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    let output = Output::new(cli.quiet, cli.verbose);
    output.info(&format!("Scanning {}...", cli.target.display()));
}
```

**Step 3: Verify it compiles**

Run: `cargo build`
Expected: Compiles successfully

**Step 4: Commit**

```bash
git add src/output.rs src/main.rs
git commit -m "feat: add progress and colored output"
```

---

### Task 12: Implement Main Orchestration

**Files:**
- Modify: `src/main.rs`

**Step 1: Implement run function**

```rust
// src/main.rs
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
```

**Step 2: Verify it compiles**

Run: `cargo build`
Expected: Compiles successfully

**Step 3: Test with a simple case**

Run: `cargo run -- --help`
Expected: Shows help text

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement main orchestration with both modes"
```

---

### Task 13: Integration Test

**Files:**
- Create: `tests/integration.rs`

**Step 1: Write integration test**

```rust
// tests/integration.rs
use std::fs::{self, File};
use std::process::Command;
use tempfile::TempDir;

fn create_test_structure(dir: &std::path::Path) {
    // Create a PSX-style game directory
    let psx = dir.join("psx").join("Final Fantasy VII");
    fs::create_dir_all(&psx).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 1).cue")).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 1).bin")).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 2).cue")).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 2).bin")).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 3).cue")).unwrap();
    File::create(psx.join("Final Fantasy VII (Disc 3).bin")).unwrap();

    // Create an Amiga-style floppy directory
    let amiga = dir.join("amiga").join("Monkey Island");
    fs::create_dir_all(&amiga).unwrap();
    File::create(amiga.join("Monkey Island (Disk 1).adf")).unwrap();
    File::create(amiga.join("Monkey Island (Disk 2).adf")).unwrap();
    File::create(amiga.join("Monkey Island (Disk 3).adf")).unwrap();
    File::create(amiga.join("Monkey Island (Disk 4).adf")).unwrap();
}

#[test]
fn test_basic_m3u_creation() {
    let dir = TempDir::new().unwrap();
    create_test_structure(dir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_m3u-emu"))
        .arg(dir.path().join("psx"))
        .output()
        .expect("Failed to run m3u-emu");

    assert!(output.status.success(), "Command failed: {:?}", output);

    // Check that m3u was created
    let m3u_path = dir.path().join("psx/Final Fantasy VII/Final Fantasy VII.m3u");
    assert!(m3u_path.exists(), "M3U file not created");

    let content = fs::read_to_string(&m3u_path).unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 3, "Should have 3 disc entries");
    assert!(lines[0].contains("Disc 1"));
    assert!(lines[1].contains("Disc 2"));
    assert!(lines[2].contains("Disc 3"));
}

#[test]
fn test_floppy_grouping() {
    let dir = TempDir::new().unwrap();
    create_test_structure(dir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_m3u-emu"))
        .arg(dir.path().join("amiga"))
        .output()
        .expect("Failed to run m3u-emu");

    assert!(output.status.success());

    // Floppy mode: all disks in one m3u
    let m3u_path = dir.path().join("amiga/Monkey Island/Monkey Island.m3u");
    assert!(m3u_path.exists());

    let content = fs::read_to_string(&m3u_path).unwrap();
    let lines: Vec<_> = content.lines().collect();
    assert_eq!(lines.len(), 4, "Should have 4 floppy entries");
}

#[test]
fn test_relative_paths() {
    let dir = TempDir::new().unwrap();
    create_test_structure(dir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_m3u-emu"))
        .arg("--relative")
        .arg(dir.path().join("psx"))
        .output()
        .expect("Failed to run m3u-emu");

    assert!(output.status.success());

    let m3u_path = dir.path().join("psx/Final Fantasy VII/Final Fantasy VII.m3u");
    let content = fs::read_to_string(&m3u_path).unwrap();

    // Relative paths should not start with /
    for line in content.lines() {
        assert!(!line.starts_with('/'), "Path should be relative: {}", line);
    }
}

#[test]
fn test_children_mode() {
    let dir = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();
    create_test_structure(dir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_m3u-emu"))
        .arg("--children")
        .arg(dir.path())
        .arg(dest.path())
        .output()
        .expect("Failed to run m3u-emu");

    assert!(output.status.success());

    // Should have created psx/ and amiga/ dirs in destination
    assert!(dest.path().join("psx").exists());
    assert!(dest.path().join("amiga").exists());

    // M3Us should be in those directories
    assert!(dest.path().join("psx/Final Fantasy VII.m3u").exists());
    assert!(dest.path().join("amiga/Monkey Island.m3u").exists());
}

#[test]
fn test_quiet_mode() {
    let dir = TempDir::new().unwrap();
    create_test_structure(dir.path());

    let output = Command::new(env!("CARGO_BIN_EXE_m3u-emu"))
        .arg("--quiet")
        .arg(dir.path().join("psx"))
        .output()
        .expect("Failed to run m3u-emu");

    assert!(output.status.success());
    // Quiet mode should have no stdout (progress goes to stderr)
    assert!(output.stdout.is_empty());
}
```

**Step 2: Run integration tests**

Run: `cargo test --test integration`
Expected: All 5 tests pass

**Step 3: Commit**

```bash
git add tests/integration.rs
git commit -m "test: add integration tests"
```

---

### Task 14: Final Polish

**Files:**
- Modify: `Cargo.toml`
- Create: `README.md`

**Step 1: Update Cargo.toml with metadata**

```toml
[package]
name = "m3u-emu"
version = "0.1.0"
edition = "2021"
description = "Generate m3u playlists for multi-disc ROM collections"
license = "MIT"
repository = "https://github.com/yourusername/m3u-emu"
keywords = ["emulation", "m3u", "playlist", "retro", "gaming"]
categories = ["command-line-utilities", "games"]

[dependencies]
clap = { version = "4", features = ["derive"] }
walkdir = "2"
regex = "1"
infer = "0.16"
indicatif = "0.17"
console = "0.15"
anyhow = "1"
once_cell = "1"
pathdiff = "0.2"

[dev-dependencies]
tempfile = "3"

[profile.release]
lto = true
strip = true
```

**Step 2: Verify release build**

Run: `cargo build --release`
Expected: Compiles successfully, binary in `target/release/m3u-emu`

**Step 3: Test the release binary**

Run: `./target/release/m3u-emu --help`
Expected: Shows help text

**Step 4: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add release profile and package metadata"
```

---

## Summary

**Total tasks:** 14
**Estimated commits:** 14

Each task follows TDD: write failing test → verify failure → implement → verify success → commit.

The implementation order builds up from primitives (types, parsing) to higher-level components (scanning, grouping) to orchestration (main). This allows each piece to be tested in isolation.
