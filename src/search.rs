use anyhow::Result;
use regex::Regex;
use std::path::Path;

use crate::models::FileEntry;

#[derive(Debug, Clone)]
pub struct SearchMode {
    pub query: String,
    pub use_regex: bool,
    pub case_sensitive: bool,
    pub search_in_contents: bool,
    pub results: Vec<SearchResult>,
    pub current_result_index: usize,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub entry: FileEntry,
    #[allow(dead_code)]
    pub match_context: Option<String>,
    #[allow(dead_code)]
    pub line_number: Option<usize>,
}

impl SearchMode {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            use_regex: false,
            case_sensitive: false,
            search_in_contents: false,
            results: Vec::new(),
            current_result_index: 0,
        }
    }

    pub fn search(&mut self, entries: &[FileEntry], _current_dir: &Path) -> Result<()> {
        self.results.clear();
        self.current_result_index = 0;

        if self.query.is_empty() {
            return Ok(());
        }

        let pattern = if self.use_regex {
            match Regex::new(&self.query) {
                Ok(regex) => Some(regex),
                Err(_) => return Ok(()), // Invalid regex, no results
            }
        } else {
            None
        };

        for entry in entries {
            if entry.name == ".." {
                continue;
            }

            // Search in filename
            let matches = if let Some(ref regex) = pattern {
                regex.is_match(&entry.name)
            } else if self.case_sensitive {
                entry.name.contains(&self.query)
            } else {
                entry
                    .name
                    .to_lowercase()
                    .contains(&self.query.to_lowercase())
            };

            if matches {
                self.results.push(SearchResult {
                    entry: entry.clone(),
                    match_context: None,
                    line_number: None,
                });
            }

            // Search in file contents if enabled and it's a text file
            if self.search_in_contents && !entry.is_dir && entry.is_accessible {
                if let Some(results) = self.search_in_file(&entry.path, &pattern)? {
                    for (line_num, context) in results {
                        self.results.push(SearchResult {
                            entry: entry.clone(),
                            match_context: Some(context),
                            line_number: Some(line_num),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn search_in_file(
        &self,
        path: &Path,
        regex: &Option<Regex>,
    ) -> Result<Option<Vec<(usize, String)>>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        // Only search in files smaller than 10MB
        if let Ok(metadata) = path.metadata() {
            if metadata.len() > 10 * 1024 * 1024 {
                return Ok(None);
            }
        }

        // Check if file is likely text
        if !self.is_text_file(path) {
            return Ok(None);
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            if let Ok(line_content) = line {
                let matches = if let Some(ref regex) = regex {
                    regex.is_match(&line_content)
                } else if self.case_sensitive {
                    line_content.contains(&self.query)
                } else {
                    line_content
                        .to_lowercase()
                        .contains(&self.query.to_lowercase())
                };

                if matches {
                    // Truncate long lines for display
                    let context = if line_content.len() > 100 {
                        format!("{}...", &line_content[..100])
                    } else {
                        line_content
                    };
                    results.push((line_num + 1, context));

                    // Limit results per file
                    if results.len() >= 5 {
                        break;
                    }
                }
            }
        }

        Ok(if results.is_empty() {
            None
        } else {
            Some(results)
        })
    }

    fn is_text_file(&self, path: &Path) -> bool {
        // Check by extension
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            matches!(
                ext.as_str(),
                "txt"
                    | "md"
                    | "rs"
                    | "toml"
                    | "yaml"
                    | "yml"
                    | "json"
                    | "js"
                    | "ts"
                    | "py"
                    | "sh"
                    | "bash"
                    | "c"
                    | "cpp"
                    | "h"
                    | "hpp"
                    | "java"
                    | "go"
                    | "rb"
                    | "php"
                    | "html"
                    | "css"
                    | "xml"
                    | "conf"
                    | "cfg"
                    | "ini"
                    | "log"
            )
        } else {
            // Check files without extension (like README, LICENSE)
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            matches!(
                filename.as_str(),
                "readme" | "license" | "makefile" | "dockerfile" | "changelog"
            )
        }
    }

    pub fn next_result(&mut self) {
        if !self.results.is_empty() {
            self.current_result_index = (self.current_result_index + 1) % self.results.len();
        }
    }

    pub fn previous_result(&mut self) {
        if !self.results.is_empty() {
            if self.current_result_index == 0 {
                self.current_result_index = self.results.len() - 1;
            } else {
                self.current_result_index -= 1;
            }
        }
    }

    pub fn toggle_regex(&mut self) {
        self.use_regex = !self.use_regex;
        // Clear results as search mode changed
        self.results.clear();
    }

    pub fn toggle_case_sensitive(&mut self) {
        self.case_sensitive = !self.case_sensitive;
        // Clear results as search mode changed
        self.results.clear();
    }

    pub fn toggle_search_contents(&mut self) {
        self.search_in_contents = !self.search_in_contents;
        // Clear results as search mode changed
        self.results.clear();
    }

    pub fn get_current_result(&self) -> Option<&SearchResult> {
        self.results.get(self.current_result_index)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.query.clear();
        self.results.clear();
        self.current_result_index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_search_mode_creation() {
        let search = SearchMode::new();
        assert!(search.query.is_empty());
        assert!(!search.use_regex);
        assert!(!search.case_sensitive);
        assert!(search.results.is_empty());
    }

    #[test]
    fn test_simple_search() {
        let mut search = SearchMode::new();
        search.query = "test".to_string();

        let entries = vec![
            FileEntry {
                name: "test.txt".to_string(),
                path: PathBuf::from("/test.txt"),
                is_dir: false,
                is_accessible: true,
                is_symlink: false,
                permissions: None,
                owner: None,
                group: None,
                uid: None,
                gid: None,
            },
            FileEntry {
                name: "other.rs".to_string(),
                path: PathBuf::from("/other.rs"),
                is_dir: false,
                is_accessible: true,
                is_symlink: false,
                permissions: None,
                owner: None,
                group: None,
                uid: None,
                gid: None,
            },
        ];

        let _ = search.search(&entries, Path::new("/"));
        assert_eq!(search.results.len(), 1);
        assert_eq!(search.results[0].entry.name, "test.txt");
    }

    #[test]
    fn test_case_insensitive_search() {
        let mut search = SearchMode::new();
        search.query = "TEST".to_string();
        search.case_sensitive = false;

        let entries = vec![FileEntry {
            name: "test.txt".to_string(),
            path: PathBuf::from("/test.txt"),
            is_dir: false,
            is_accessible: true,
            is_symlink: false,
            permissions: None,
            owner: None,
            group: None,
            uid: None,
            gid: None,
        }];

        let _ = search.search(&entries, Path::new("/"));
        assert_eq!(search.results.len(), 1);
    }

    #[test]
    fn test_regex_search() {
        let mut search = SearchMode::new();
        search.query = r"^test.*\.txt$".to_string();
        search.use_regex = true;

        let entries = vec![
            FileEntry {
                name: "test123.txt".to_string(),
                path: PathBuf::from("/test123.txt"),
                is_dir: false,
                is_accessible: true,
                is_symlink: false,
                permissions: None,
                owner: None,
                group: None,
                uid: None,
                gid: None,
            },
            FileEntry {
                name: "test.rs".to_string(),
                path: PathBuf::from("/test.rs"),
                is_dir: false,
                is_accessible: true,
                is_symlink: false,
                permissions: None,
                owner: None,
                group: None,
                uid: None,
                gid: None,
            },
        ];

        let _ = search.search(&entries, Path::new("/"));
        assert_eq!(search.results.len(), 1);
        assert_eq!(search.results[0].entry.name, "test123.txt");
    }

    #[test]
    fn test_navigation() {
        let mut search = SearchMode::new();

        // Add mock results
        for i in 0..3 {
            search.results.push(SearchResult {
                entry: FileEntry {
                    name: format!("file{}.txt", i),
                    path: PathBuf::from(format!("/file{}.txt", i)),
                    is_dir: false,
                    is_accessible: true,
                    is_symlink: false,
                    permissions: None,
                    owner: None,
                    group: None,
                    uid: None,
                    gid: None,
                },
                match_context: None,
                line_number: None,
            });
        }

        assert_eq!(search.current_result_index, 0);

        search.next_result();
        assert_eq!(search.current_result_index, 1);

        search.next_result();
        assert_eq!(search.current_result_index, 2);

        search.next_result();
        assert_eq!(search.current_result_index, 0); // Wraps around

        search.previous_result();
        assert_eq!(search.current_result_index, 2); // Wraps backward
    }
}
