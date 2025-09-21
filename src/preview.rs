use anyhow::Result;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct FilePreview {
    pub content: PreviewContent,
    pub file_info: FileInfo,
    pub scroll_offset: usize,
}

#[derive(Debug, Clone)]
pub enum PreviewContent {
    Text(Vec<String>),
    Binary(Vec<u8>),
    Image(ImageInfo),
    Directory(Vec<String>),
    Error(String),
    #[allow(dead_code)]
    Empty,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub size: u64,
    #[allow(dead_code)]
    pub modified: Option<std::time::SystemTime>,
    pub permissions: Option<u32>,
    pub mime_type: String,
    #[allow(dead_code)]
    pub line_count: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    #[allow(dead_code)]
    pub format: String,
    #[allow(dead_code)]
    pub dimensions: Option<(u32, u32)>,
    pub ascii_art: Option<String>,
}

impl FilePreview {
    pub fn new(path: &Path, max_lines: usize) -> Result<Self> {
        let metadata = fs::metadata(path)?;

        let file_info = FileInfo {
            size: metadata.len(),
            modified: metadata.modified().ok(),
            permissions: {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    Some(metadata.permissions().mode())
                }
                #[cfg(not(unix))]
                {
                    None
                }
            },
            mime_type: Self::detect_mime_type(path),
            line_count: None,
        };

        let content = if metadata.is_dir() {
            Self::preview_directory(path, max_lines)?
        } else {
            Self::preview_file(path, max_lines, metadata.len())?
        };

        Ok(Self {
            content,
            file_info,
            scroll_offset: 0,
        })
    }

    fn detect_mime_type(path: &Path) -> String {
        if path.is_dir() {
            return "inode/directory".to_string();
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            // Text files
            "txt" | "md" | "markdown" => "text/plain",
            "rs" => "text/x-rust",
            "py" => "text/x-python",
            "js" | "mjs" => "text/javascript",
            "ts" => "text/typescript",
            "java" => "text/x-java",
            "c" => "text/x-c",
            "cpp" | "cc" | "cxx" => "text/x-c++",
            "h" | "hpp" => "text/x-c-header",
            "go" => "text/x-go",
            "rb" => "text/x-ruby",
            "php" => "text/x-php",
            "sh" | "bash" => "text/x-shellscript",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "xml" => "text/xml",
            "json" => "application/json",
            "yaml" | "yml" => "text/x-yaml",
            "toml" => "text/x-toml",
            "ini" | "cfg" | "conf" => "text/x-ini",
            "log" => "text/x-log",

            // Images
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "bmp" => "image/bmp",
            "svg" => "image/svg+xml",
            "ico" => "image/x-icon",
            "webp" => "image/webp",

            // Archives
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" | "gzip" => "application/gzip",
            "bz2" => "application/x-bzip2",
            "xz" => "application/x-xz",
            "7z" => "application/x-7z-compressed",
            "rar" => "application/x-rar",

            // Documents
            "pdf" => "application/pdf",
            "doc" | "docx" => "application/msword",
            "xls" | "xlsx" => "application/vnd.ms-excel",
            "ppt" | "pptx" => "application/vnd.ms-powerpoint",

            // Media
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "ogg" => "audio/ogg",
            "mp4" => "video/mp4",
            "avi" => "video/x-msvideo",
            "mkv" => "video/x-matroska",

            _ => "application/octet-stream",
        }
        .to_string()
    }

    fn preview_file(path: &Path, max_lines: usize, file_size: u64) -> Result<PreviewContent> {
        // Don't preview files larger than 10MB
        if file_size > 10 * 1024 * 1024 {
            return Ok(PreviewContent::Error(
                "File too large to preview".to_string(),
            ));
        }

        let mime_type = Self::detect_mime_type(path);

        if mime_type.starts_with("text/")
            || mime_type == "application/json"
            || Self::is_text_file_by_content(path)?
        {
            Self::preview_text_file(path, max_lines)
        } else if mime_type.starts_with("image/") {
            Self::preview_image_file(path)
        } else {
            Self::preview_binary_file(path)
        }
    }

    fn is_text_file_by_content(path: &Path) -> Result<bool> {
        let mut file = File::open(path)?;
        let mut buffer = [0; 512];
        let bytes_read = file.read(&mut buffer)?;

        // Check if file contains null bytes (binary indicator)
        for &b in buffer.iter().take(bytes_read) {
            if b == 0 {
                return Ok(false);
            }
            // Check for other non-text bytes
            if b < 0x20 && !matches!(b, 0x09 | 0x0A | 0x0D) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn preview_text_file(path: &Path, max_lines: usize) -> Result<PreviewContent> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut lines = Vec::new();
        let mut _line_count = 0;

        for line_result in reader.lines().take(max_lines) {
            match line_result {
                Ok(line) => {
                    // Replace tabs with spaces for better display
                    let line = line.replace('\t', "    ");
                    lines.push(line);
                    _line_count += 1;
                }
                Err(_) => {
                    // Not a valid UTF-8 file
                    return Self::preview_binary_file(path);
                }
            }
        }

        Ok(PreviewContent::Text(lines))
    }

    fn preview_binary_file(path: &Path) -> Result<PreviewContent> {
        let mut file = File::open(path)?;
        let mut buffer = vec![0; 256]; // First 256 bytes for hex preview
        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        Ok(PreviewContent::Binary(buffer))
    }

    fn preview_image_file(path: &Path) -> Result<PreviewContent> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let image_info = ImageInfo {
            format: ext.clone(),
            dimensions: None, // Would need image crate to get actual dimensions
            ascii_art: Self::generate_ascii_placeholder(&ext),
        };

        Ok(PreviewContent::Image(image_info))
    }

    fn generate_ascii_placeholder(format: &str) -> Option<String> {
        let art = match format {
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" => {
                r#"
    ┌───────────────┐
    │   🖼️ IMAGE    │
    │               │
    │   [Preview    │
    │    not yet    │
    │   available]  │
    │               │
    └───────────────┘"#
            }
            "svg" => {
                r#"
    ┌───────────────┐
    │   📐 SVG      │
    │               │
    │  <Vector>     │
    │   Graphics    │
    │               │
    └───────────────┘"#
            }
            _ => return None,
        };

        Some(art.to_string())
    }

    fn preview_directory(path: &Path, max_entries: usize) -> Result<PreviewContent> {
        let mut entries = Vec::new();
        let mut count = 0;

        if let Ok(read_dir) = fs::read_dir(path) {
            for entry in read_dir.flatten() {
                if count >= max_entries {
                    entries.push("...".to_string());
                    break;
                }

                let file_name = entry.file_name().to_string_lossy().to_string();
                let file_type = if entry.path().is_dir() {
                    "📁"
                } else {
                    "📄"
                };

                entries.push(format!("{} {}", file_type, file_name));
                count += 1;
            }
        }

        if entries.is_empty() {
            entries.push("(empty directory)".to_string());
        }

        Ok(PreviewContent::Directory(entries))
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        let max_offset = match &self.content {
            PreviewContent::Text(text) => text.len().saturating_sub(1),
            PreviewContent::Directory(entries) => entries.len().saturating_sub(1),
            _ => 0,
        };

        self.scroll_offset = (self.scroll_offset + lines).min(max_offset);
    }

    pub fn format_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }

    pub fn format_permissions(mode: u32) -> String {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_mime_type_detection() {
        assert_eq!(
            FilePreview::detect_mime_type(Path::new("test.txt")),
            "text/plain"
        );
        assert_eq!(
            FilePreview::detect_mime_type(Path::new("code.rs")),
            "text/x-rust"
        );
        assert_eq!(
            FilePreview::detect_mime_type(Path::new("image.png")),
            "image/png"
        );
        assert_eq!(
            FilePreview::detect_mime_type(Path::new("archive.zip")),
            "application/zip"
        );
        assert_eq!(
            FilePreview::detect_mime_type(Path::new("unknown.xyz")),
            "application/octet-stream"
        );
    }

    #[test]
    fn test_format_size() {
        assert_eq!(FilePreview::format_size(512), "512 B");
        assert_eq!(FilePreview::format_size(1024), "1.00 KB");
        assert_eq!(FilePreview::format_size(1536), "1.50 KB");
        assert_eq!(FilePreview::format_size(1048576), "1.00 MB");
        assert_eq!(FilePreview::format_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_permissions() {
        assert_eq!(FilePreview::format_permissions(0o755), "rwxr-xr-x");
        assert_eq!(FilePreview::format_permissions(0o644), "rw-r--r--");
        assert_eq!(FilePreview::format_permissions(0o600), "rw-------");
        assert_eq!(FilePreview::format_permissions(0o777), "rwxrwxrwx");
        assert_eq!(FilePreview::format_permissions(0o000), "---------");
    }
}
