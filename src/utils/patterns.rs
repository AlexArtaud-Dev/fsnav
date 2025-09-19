use regex::Regex;

/// Match a pattern against a string
/// Supports:
/// - Glob patterns with * (e.g., "*.txt", "file*")
/// - Regex patterns (automatically detected)
/// - Simple substring matching
pub fn match_pattern(pattern: &str, text: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }

    // Check if it's a glob pattern
    if pattern.contains('*') {
        let regex_pattern = pattern
            .replace('.', r"\.")
            .replace('*', ".*")
            .replace('?', ".");

        if let Ok(regex) = Regex::new(&format!("^{}$", regex_pattern)) {
            return regex.is_match(text);
        }
    }

    // Try as regex
    if let Ok(regex) = Regex::new(pattern) {
        return regex.is_match(text);
    }

    // Fall back to substring matching
    text.contains(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glob_patterns() {
        assert!(match_pattern("*.txt", "file.txt"));
        assert!(!match_pattern("*.txt", "file.md"));
        assert!(match_pattern("file*", "file123"));
        assert!(match_pattern("*test*", "mytestfile"));
    }

    #[test]
    fn test_regex_patterns() {
        assert!(match_pattern(r"^\d+$", "123"));
        assert!(!match_pattern(r"^\d+$", "abc"));
        assert!(match_pattern(r"test\d+", "test123"));
    }

    #[test]
    fn test_substring_matching() {
        assert!(match_pattern("test", "mytestfile"));
        assert!(!match_pattern("test", "myfile"));
    }
}
