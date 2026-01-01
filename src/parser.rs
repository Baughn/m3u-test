// Parser module for extracting disc numbers from filenames

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

static NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d+").unwrap());

/// Extract a number from a string containing digits, words, or letters
pub fn extract_number(s: &str) -> Option<u32> {
    // First try to find a numeric digit sequence
    if let Some(m) = NUMBER_REGEX.find(s) {
        if let Ok(n) = m.as_str().parse::<u32>() {
            return Some(n);
        }
    }

    // Split into words, try to convert
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
}
