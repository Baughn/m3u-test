use std::path::PathBuf;

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
