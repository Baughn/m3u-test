use crate::types::{MediaFile, MediaType};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

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
}
