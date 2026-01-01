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
