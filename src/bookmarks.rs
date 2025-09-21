use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub name: String,
    pub path: PathBuf,
    pub shortcut: Option<char>,
    pub created_at: std::time::SystemTime,
    pub last_accessed: Option<std::time::SystemTime>,
    pub access_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarksManager {
    bookmarks: Vec<Bookmark>,
    shortcuts: HashMap<char, usize>, // Maps shortcut to bookmark index
    config_path: PathBuf,
}

impl BookmarksManager {
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let config_path = config_dir.join("bookmarks.json");

        let mut manager = Self {
            bookmarks: Vec::new(),
            shortcuts: HashMap::new(),
            config_path,
        };

        // Load existing bookmarks if file exists
        if manager.config_path.exists() {
            manager.load()?;
        } else {
            // Create default bookmarks
            manager.create_default_bookmarks();
            manager.save()?;
        }

        Ok(manager)
    }

    fn get_config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to get home directory")?;
        let config_dir = home.join(".config").join("fsnav");

        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir)
    }

    fn create_default_bookmarks(&mut self) {
        // Add common directories as default bookmarks
        if let Some(home) = dirs::home_dir() {
            self.add_bookmark_internal(
                "Home".to_string(),
                home.clone(),
                Some('h'),
            );

            let downloads = home.join("Downloads");
            if downloads.exists() {
                self.add_bookmark_internal(
                    "Downloads".to_string(),
                    downloads,
                    Some('d'),
                );
            }

            let documents = home.join("Documents");
            if documents.exists() {
                self.add_bookmark_internal(
                    "Documents".to_string(),
                    documents,
                    Some('o'),
                );
            }

            let desktop = home.join("Desktop");
            if desktop.exists() {
                self.add_bookmark_internal(
                    "Desktop".to_string(),
                    desktop,
                    Some('k'),
                );
            }
        }

        // Add root directory
        self.add_bookmark_internal(
            "Root".to_string(),
            PathBuf::from("/"),
            Some('r'),
        );

        // Add common system directories
        if Path::new("/usr/local").exists() {
            self.add_bookmark_internal(
                "Local".to_string(),
                PathBuf::from("/usr/local"),
                Some('l'),
            );
        }

        if Path::new("/etc").exists() {
            self.add_bookmark_internal(
                "Config".to_string(),
                PathBuf::from("/etc"),
                Some('e'),
            );
        }

        if Path::new("/tmp").exists() {
            self.add_bookmark_internal(
                "Temp".to_string(),
                PathBuf::from("/tmp"),
                Some('t'),
            );
        }
    }

    fn add_bookmark_internal(&mut self, name: String, path: PathBuf, shortcut: Option<char>) {
        let bookmark = Bookmark {
            name,
            path,
            shortcut,
            created_at: std::time::SystemTime::now(),
            last_accessed: None,
            access_count: 0,
        };

        let index = self.bookmarks.len();
        self.bookmarks.push(bookmark);

        if let Some(key) = shortcut {
            self.shortcuts.insert(key, index);
        }
    }

    pub fn add_bookmark(&mut self, name: String, path: PathBuf, shortcut: Option<char>) -> Result<()> {
        // Check if path exists
        if !path.exists() {
            return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
        }

        // Check if bookmark already exists
        if self.bookmarks.iter().any(|b| b.path == path) {
            return Err(anyhow::anyhow!("Bookmark already exists for this path"));
        }

        // Check if shortcut is already taken
        if let Some(key) = shortcut {
            if self.shortcuts.contains_key(&key) {
                return Err(anyhow::anyhow!("Shortcut '{}' is already in use", key));
            }
        }

        self.add_bookmark_internal(name, path, shortcut);
        self.save()?;
        Ok(())
    }

    pub fn remove_bookmark(&mut self, index: usize) -> Result<()> {
        if index >= self.bookmarks.len() {
            return Err(anyhow::anyhow!("Invalid bookmark index"));
        }

        let bookmark = self.bookmarks.remove(index);

        // Remove shortcut if it exists
        if let Some(key) = bookmark.shortcut {
            self.shortcuts.remove(&key);
        }

        // Update shortcut indices for remaining bookmarks
        self.shortcuts = self.shortcuts.iter()
            .map(|(&k, &v)| {
                if v > index {
                    (k, v - 1)
                } else {
                    (k, v)
                }
            })
            .collect();

        self.save()?;
        Ok(())
    }

    pub fn rename_bookmark(&mut self, index: usize, new_name: String) -> Result<()> {
        if index >= self.bookmarks.len() {
            return Err(anyhow::anyhow!("Invalid bookmark index"));
        }

        self.bookmarks[index].name = new_name;
        self.save()?;
        Ok(())
    }

    pub fn update_shortcut(&mut self, index: usize, new_shortcut: Option<char>) -> Result<()> {
        if index >= self.bookmarks.len() {
            return Err(anyhow::anyhow!("Invalid bookmark index"));
        }

        // Remove old shortcut
        if let Some(old_key) = self.bookmarks[index].shortcut {
            self.shortcuts.remove(&old_key);
        }

        // Check if new shortcut is already taken
        if let Some(key) = new_shortcut {
            if self.shortcuts.contains_key(&key) {
                return Err(anyhow::anyhow!("Shortcut '{}' is already in use", key));
            }
            self.shortcuts.insert(key, index);
        }

        self.bookmarks[index].shortcut = new_shortcut;
        self.save()?;
        Ok(())
    }

    pub fn get_bookmark_by_shortcut(&mut self, shortcut: char) -> Option<&Bookmark> {
        if let Some(&index) = self.shortcuts.get(&shortcut) {
            if let Some(bookmark) = self.bookmarks.get_mut(index) {
                bookmark.last_accessed = Some(std::time::SystemTime::now());
                bookmark.access_count += 1;
                let _ = self.save(); // Ignore save errors for access updates
                return self.bookmarks.get(index);
            }
        }
        None
    }

    pub fn get_bookmark_by_index(&mut self, index: usize) -> Option<&Bookmark> {
        if let Some(bookmark) = self.bookmarks.get_mut(index) {
            bookmark.last_accessed = Some(std::time::SystemTime::now());
            bookmark.access_count += 1;
            let _ = self.save(); // Ignore save errors for access updates
            return self.bookmarks.get(index);
        }
        None
    }

    pub fn list_bookmarks(&self) -> &[Bookmark] {
        &self.bookmarks
    }

    pub fn find_bookmark_by_path(&self, path: &Path) -> Option<usize> {
        self.bookmarks.iter().position(|b| b.path == path)
    }

    pub fn sort_by_frequency(&mut self) {
        self.bookmarks.sort_by(|a, b| b.access_count.cmp(&a.access_count));

        // Rebuild shortcuts map
        self.shortcuts.clear();
        for (index, bookmark) in self.bookmarks.iter().enumerate() {
            if let Some(key) = bookmark.shortcut {
                self.shortcuts.insert(key, index);
            }
        }

        let _ = self.save();
    }

    pub fn sort_by_name(&mut self) {
        self.bookmarks.sort_by(|a, b| a.name.cmp(&b.name));

        // Rebuild shortcuts map
        self.shortcuts.clear();
        for (index, bookmark) in self.bookmarks.iter().enumerate() {
            if let Some(key) = bookmark.shortcut {
                self.shortcuts.insert(key, index);
            }
        }

        let _ = self.save();
    }

    pub fn get_available_shortcuts(&self) -> Vec<char> {
        let mut available = Vec::new();
        for c in 'a'..='z' {
            if !self.shortcuts.contains_key(&c) {
                available.push(c);
            }
        }
        for c in '0'..='9' {
            if !self.shortcuts.contains_key(&c) {
                available.push(c);
            }
        }
        available
    }

    fn load(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.config_path)?;
        let data: SavedBookmarks = serde_json::from_str(&content)?;

        self.bookmarks = data.bookmarks;

        // Rebuild shortcuts map
        self.shortcuts.clear();
        for (index, bookmark) in self.bookmarks.iter().enumerate() {
            if let Some(key) = bookmark.shortcut {
                self.shortcuts.insert(key, index);
            }
        }

        Ok(())
    }

    fn save(&self) -> Result<()> {
        let data = SavedBookmarks {
            version: 1,
            bookmarks: self.bookmarks.clone(),
        };

        let json = serde_json::to_string_pretty(&data)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }

    pub fn export_to_file(&self, path: &Path) -> Result<()> {
        let data = SavedBookmarks {
            version: 1,
            bookmarks: self.bookmarks.clone(),
        };

        let json = serde_json::to_string_pretty(&data)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn import_from_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let data: SavedBookmarks = serde_json::from_str(&content)?;

        // Merge with existing bookmarks
        for bookmark in data.bookmarks {
            // Skip if path already bookmarked
            if !self.bookmarks.iter().any(|b| b.path == bookmark.path) {
                let index = self.bookmarks.len();

                // Find new shortcut if current one is taken
                let shortcut = if let Some(key) = bookmark.shortcut {
                    if self.shortcuts.contains_key(&key) {
                        None // Will need to assign manually
                    } else {
                        Some(key)
                    }
                } else {
                    None
                };

                self.bookmarks.push(Bookmark {
                    shortcut,
                    ..bookmark
                });

                if let Some(key) = shortcut {
                    self.shortcuts.insert(key, index);
                }
            }
        }

        self.save()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct SavedBookmarks {
    version: u32,
    bookmarks: Vec<Bookmark>,
}

// Directory for home_dir fallback
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .ok()
            .map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bookmark_operations() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let mut manager = BookmarksManager::new().unwrap();

        // Test adding bookmark
        let test_path = temp_dir.path().join("test");
        fs::create_dir(&test_path).unwrap();

        manager.add_bookmark(
            "Test".to_string(),
            test_path.clone(),
            Some('x')
        ).unwrap();

        // Test finding by shortcut
        assert!(manager.get_bookmark_by_shortcut('x').is_some());

        // Test finding by path
        let index = manager.find_bookmark_by_path(&test_path);
        assert!(index.is_some());

        // Test removing bookmark
        manager.remove_bookmark(index.unwrap()).unwrap();
        assert!(manager.get_bookmark_by_shortcut('x').is_none());
    }

    #[test]
    fn test_shortcut_conflicts() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let mut manager = BookmarksManager::new().unwrap();

        let path1 = temp_dir.path().join("test1");
        let path2 = temp_dir.path().join("test2");
        fs::create_dir(&path1).unwrap();
        fs::create_dir(&path2).unwrap();

        manager.add_bookmark("Test1".to_string(), path1, Some('x')).unwrap();

        // Should fail due to shortcut conflict
        let result = manager.add_bookmark("Test2".to_string(), path2, Some('x'));
        assert!(result.is_err());
    }
}