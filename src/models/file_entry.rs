use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_accessible: bool,
    pub is_symlink: bool,
    pub permissions: Option<u32>,
    pub owner: Option<String>,
    pub group: Option<String>,
    #[allow(dead_code)]
    pub uid: Option<u32>,
    #[allow(dead_code)]
    pub gid: Option<u32>,
}

impl FileEntry {
    pub fn display_name(&self) -> String {
        let icon = if self.is_symlink {
            "🔗"
        } else if self.is_dir {
            "📁"
        } else {
            "📄"
        };

        let name = if self.is_dir && !self.is_symlink {
            format!("{}/", self.name)
        } else {
            self.name.clone()
        };

        format!("{} {}", icon, name)
    }

    pub fn permissions_string(&self) -> String {
        match self.permissions {
            Some(mode) => {
                let user = (mode >> 6) & 0b111;
                let group = (mode >> 3) & 0b111;
                let other = mode & 0b111;

                let to_rwx = |p: u32| {
                    format!(
                        "{}{}{}",
                        if p & 4 != 0 { "r" } else { "-" },
                        if p & 2 != 0 { "w" } else { "-" },
                        if p & 1 != 0 { "x" } else { "-" }
                    )
                };

                format!("{}{}{}", to_rwx(user), to_rwx(group), to_rwx(other))
            }
            None => "---------".to_string(),
        }
    }

    pub fn ownership_string(&self) -> String {
        format!(
            "{} {}",
            self.owner.as_ref().unwrap_or(&"-".to_string()),
            self.group.as_ref().unwrap_or(&"-".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_entry_display() {
        let dir_entry = FileEntry {
            name: "test_dir".to_string(),
            path: PathBuf::from("/test/test_dir"),
            is_dir: true,
            is_accessible: true,
            is_symlink: false,
            permissions: Some(0o755),
            owner: Some("user".to_string()),
            group: Some("group".to_string()),
            uid: Some(1000),
            gid: Some(1000),
        };
        assert_eq!(dir_entry.display_name(), "📁 test_dir/");

        let file_entry = FileEntry {
            name: "test.txt".to_string(),
            path: PathBuf::from("/test/test.txt"),
            is_dir: false,
            is_accessible: true,
            is_symlink: false,
            permissions: Some(0o644),
            owner: Some("user".to_string()),
            group: Some("group".to_string()),
            uid: Some(1000),
            gid: Some(1000),
        };
        assert_eq!(file_entry.display_name(), "📄 test.txt");
    }

    #[test]
    fn test_permissions_string() {
        let entry = FileEntry {
            name: "test".to_string(),
            path: PathBuf::from("/test"),
            is_dir: false,
            is_accessible: true,
            is_symlink: false,
            permissions: Some(0o755),
            owner: None,
            group: None,
            uid: None,
            gid: None,
        };
        assert_eq!(entry.permissions_string(), "rwxr-xr-x");
    }
}
