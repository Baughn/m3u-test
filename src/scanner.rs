// Scanner module for finding media files in directories

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
    use std::fs::File;
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
