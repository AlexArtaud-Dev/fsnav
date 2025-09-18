use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use regex::Regex;
use std::{
    collections::HashSet,
    env, fs,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

mod permissions;
use permissions::ChmodInterface;

#[derive(Debug, Clone)]
struct FileEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    is_accessible: bool,
    is_symlink: bool,
    permissions: Option<u32>,
    owner: Option<String>,
    group: Option<String>,
}

impl FileEntry {
    fn display_name(&self) -> String {
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

    fn permissions_string(&self) -> String {
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
}

#[derive(Debug, PartialEq)]
enum NavigatorMode {
    Browse,
    Select,
    ChmodInterface,
    PatternSelect,
}

struct Navigator {
    current_dir: PathBuf,
    entries: Vec<FileEntry>,
    selected_index: usize,
    selected_items: HashSet<usize>,
    scroll_offset: usize,
    terminal_height: u16,
    mode: NavigatorMode,
    is_root: bool,
    pattern_input: String,
    chmod_interface: Option<ChmodInterface>,
    status_message: Option<String>,
}

impl Navigator {
    fn new() -> Result<Self> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        let is_root = unsafe { libc::geteuid() } == 0;

        let mut nav = Self {
            current_dir: current_dir.clone(),
            entries: Vec::new(),
            selected_index: 0,
            selected_items: HashSet::new(),
            scroll_offset: 0,
            terminal_height: terminal::size()?.1,
            mode: NavigatorMode::Browse,
            is_root,
            pattern_input: String::new(),
            chmod_interface: None,
            status_message: None,
        };
        nav.load_directory(&current_dir)?;
        Ok(nav)
    }

    fn load_directory(&mut self, path: &Path) -> Result<()> {
        self.entries.clear();
        self.selected_index = 0;
        self.selected_items.clear();
        self.scroll_offset = 0;

        // Add parent directory entry if not at root
        if let Some(parent) = path.parent() {
            if parent != path {
                self.entries.push(FileEntry {
                    name: "..".to_string(),
                    path: parent.to_path_buf(),
                    is_dir: true,
                    is_accessible: true,
                    is_symlink: false,
                    permissions: None,
                    owner: None,
                    group: None,
                });
            }
        }

        // Read directory entries
        match fs::read_dir(path) {
            Ok(read_dir) => {
                let mut dir_entries = Vec::new();
                let mut file_entries = Vec::new();

                for entry in read_dir.flatten() {
                    let path = entry.path();
                    let metadata = entry.metadata();
                    let symlink_metadata = entry.path().symlink_metadata();

                    let is_symlink = symlink_metadata
                        .as_ref()
                        .map(|m| m.file_type().is_symlink())
                        .unwrap_or(false);

                    let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                    let is_accessible = metadata.is_ok();

                    let permissions = metadata.as_ref().ok().map(|m| m.permissions().mode());

                    // Get owner and group info (simplified for now)
                    let (owner, group) = if cfg!(unix) {
                        Self::get_owner_group(&path)
                    } else {
                        (None, None)
                    };

                    let name = entry.file_name().to_string_lossy().to_string();

                    // Skip hidden files on Unix-like systems
                    #[cfg(unix)]
                    if name.starts_with('.') && name != ".." {
                        continue;
                    }

                    let file_entry = FileEntry {
                        name,
                        path,
                        is_dir,
                        is_accessible,
                        is_symlink,
                        permissions,
                        owner,
                        group,
                    };

                    if is_dir {
                        dir_entries.push(file_entry);
                    } else {
                        file_entries.push(file_entry);
                    }
                }

                // Sort directories and files separately
                dir_entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                file_entries.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                // Add sorted entries (directories first)
                self.entries.extend(dir_entries);
                self.entries.extend(file_entries);
            }
            Err(e) => {
                // If directory is not accessible, show error but don't crash
                self.entries.push(FileEntry {
                    name: format!("⚠️  Error: {}", e),
                    path: path.to_path_buf(),
                    is_dir: false,
                    is_accessible: false,
                    is_symlink: false,
                    permissions: None,
                    owner: None,
                    group: None,
                });
            }
        }

        self.current_dir = path.to_path_buf();
        Ok(())
    }

    fn get_owner_group(path: &Path) -> (Option<String>, Option<String>) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;

            if let Ok(metadata) = path.metadata() {
                let uid = metadata.uid();
                let gid = metadata.gid();

                // Get username from uid
                let owner = unsafe {
                    let pw = libc::getpwuid(uid);
                    if !pw.is_null() {
                        let name = std::ffi::CStr::from_ptr((*pw).pw_name);
                        name.to_string_lossy().to_string()
                    } else {
                        uid.to_string()
                    }
                };

                // Get group name from gid
                let group = unsafe {
                    let gr = libc::getgrgid(gid);
                    if !gr.is_null() {
                        let name = std::ffi::CStr::from_ptr((*gr).gr_name);
                        name.to_string_lossy().to_string()
                    } else {
                        gid.to_string()
                    }
                };

                return (Some(owner), Some(group));
            }
        }

        (None, None)
    }

    fn navigate_to_selected(&mut self) -> Result<()> {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir && entry.is_accessible {
                let new_path = entry.path.clone();
                self.load_directory(&new_path)?;
            }
        }
        Ok(())
    }

    fn navigate_up(&mut self) -> Result<()> {
        if let Some(parent) = self.current_dir.parent() {
            let parent_path = parent.to_path_buf();
            self.load_directory(&parent_path)?;
        }
        Ok(())
    }

    fn move_selection_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.adjust_scroll();
        }
    }

    fn move_selection_down(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
            self.adjust_scroll();
        }
    }

    fn toggle_selection(&mut self) {
        // Don't allow selecting ".."
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.name != ".." {
                if self.selected_items.contains(&self.selected_index) {
                    self.selected_items.remove(&self.selected_index);
                } else {
                    self.selected_items.insert(self.selected_index);
                }
            }
        }
    }

    fn select_by_pattern(&mut self) {
        if self.pattern_input.is_empty() {
            return;
        }

        self.selected_items.clear();

        // Handle different pattern types
        if self.pattern_input.contains('*') {
            // Simple glob pattern: "ali*" means starts with "ali"
            let prefix = self.pattern_input.trim_end_matches('*');
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.name != ".." && entry.name.starts_with(prefix) {
                    self.selected_items.insert(i);
                }
            }
            self.status_message = Some(format!(
                "Selected {} items starting with '{}'",
                self.selected_items.len(),
                prefix
            ));
        } else if let Ok(regex) = Regex::new(&self.pattern_input) {
            // Try as regex if no glob pattern
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.name != ".." && regex.is_match(&entry.name) {
                    self.selected_items.insert(i);
                }
            }
            self.status_message = Some(format!(
                "Selected {} items matching regex '{}'",
                self.selected_items.len(),
                self.pattern_input
            ));
        } else {
            // Fall back to simple substring matching
            for (i, entry) in self.entries.iter().enumerate() {
                if entry.name != ".." && entry.name.contains(&self.pattern_input) {
                    self.selected_items.insert(i);
                }
            }
            self.status_message = Some(format!(
                "Selected {} items containing '{}'",
                self.selected_items.len(),
                self.pattern_input
            ));
        }

        self.pattern_input.clear();
    }

    fn open_chmod_interface(&mut self) {
        if !self.is_root {
            self.status_message = Some("⚠️  Chmod interface requires root privileges".to_string());
            return;
        }

        let selected_paths: Vec<PathBuf> = if self.selected_items.is_empty() {
            // Use currently highlighted item
            if let Some(entry) = self.entries.get(self.selected_index) {
                if entry.name != ".." {
                    vec![entry.path.clone()]
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            // Use all selected items
            self.selected_items
                .iter()
                .filter_map(|&i| self.entries.get(i))
                .filter(|e| e.name != "..")
                .map(|e| e.path.clone())
                .collect()
        };

        if selected_paths.is_empty() {
            self.status_message = Some("No items selected for chmod".to_string());
            return;
        }

        self.chmod_interface = Some(ChmodInterface::new(selected_paths));
        self.mode = NavigatorMode::ChmodInterface;
    }

    fn adjust_scroll(&mut self) {
        let visible_area = (self.terminal_height as usize).saturating_sub(5);

        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_area {
            self.scroll_offset = self.selected_index.saturating_sub(visible_area - 1);
        }
    }

    fn render(&self) -> Result<()> {
        if self.mode == NavigatorMode::ChmodInterface {
            if let Some(ref chmod) = self.chmod_interface {
                return chmod.render();
            }
        }

        let mut stdout = io::stdout();
        let (terminal_width, terminal_height) = terminal::size()?;

        // Clear screen
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        // Draw header with breadcrumb
        let header_text = if self.is_root {
            format!(" 📂 {} [ROOT MODE]", self.current_dir.display())
        } else {
            format!(" 📂 {}", self.current_dir.display())
        };

        execute!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            Print(" ".repeat(terminal_width as usize)),
            MoveTo(0, 0),
            Print(&header_text),
            ResetColor
        )?;

        // Mode indicator
        let mode_text = match self.mode {
            NavigatorMode::Browse => "BROWSE".to_string(),
            NavigatorMode::Select => "SELECT (Space: toggle, Enter: confirm)".to_string(),
            NavigatorMode::PatternSelect => format!("PATTERN: {}_", self.pattern_input),
            _ => String::new(),
        };

        if !mode_text.is_empty() {
            execute!(
                stdout,
                MoveTo(0, 1),
                SetForegroundColor(Color::Yellow),
                Print(format!(" Mode: {} ", mode_text)),
                ResetColor
            )?;
        }

        // Draw file list
        let list_start = 3;
        let visible_area = (terminal_height as usize).saturating_sub(5);
        let end_index = (self.scroll_offset + visible_area).min(self.entries.len());

        for (i, entry) in self.entries[self.scroll_offset..end_index]
            .iter()
            .enumerate()
        {
            let row = (list_start + i) as u16;
            execute!(stdout, MoveTo(0, row))?;

            let display_index = self.scroll_offset + i;
            let is_selected = self.selected_items.contains(&display_index);
            let is_highlighted = display_index == self.selected_index;

            // Selection indicator
            let selection_marker = if is_selected { "[✓]" } else { "[ ]" };

            if is_highlighted {
                execute!(
                    stdout,
                    SetBackgroundColor(Color::DarkGrey),
                    SetForegroundColor(Color::White)
                )?;
            }

            // Show selection checkbox in select mode
            if self.mode == NavigatorMode::Select {
                execute!(stdout, Print(format!(" {} ", selection_marker)))?;
            }

            // Entry name
            let display_str = if is_highlighted {
                format!(" > {}", entry.display_name())
            } else {
                format!("   {}", entry.display_name())
            };

            let color = if !entry.is_accessible {
                Color::DarkRed
            } else if entry.is_dir {
                Color::Cyan
            } else if entry.is_symlink {
                Color::Magenta
            } else {
                Color::White
            };

            execute!(stdout, SetForegroundColor(color), Print(&display_str))?;

            // Show permissions if in select mode and root
            if self.mode == NavigatorMode::Select && self.is_root {
                let perms = entry.permissions_string();
                let owner_group = format!(
                    " {} {} {}",
                    perms,
                    entry.owner.as_ref().unwrap_or(&"-".to_string()),
                    entry.group.as_ref().unwrap_or(&"-".to_string())
                );
                execute!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print(&owner_group)
                )?;
            }

            if is_highlighted {
                // Calculate actual content length more accurately
                let content_len = display_str.len()
                    + if self.mode == NavigatorMode::Select {
                        4
                    } else {
                        0
                    }
                    + if self.mode == NavigatorMode::Select && self.is_root {
                        20 + // permissions
                        entry.owner.as_ref().map(|o| o.len()).unwrap_or(1) + 1 +
                        entry.group.as_ref().map(|g| g.len()).unwrap_or(1) + 1
                    } else {
                        0
                    };

                // Only fill up to terminal width to prevent wrapping
                let padding = (terminal_width as usize)
                    .saturating_sub(content_len)
                    .min(terminal_width as usize);
                execute!(stdout, Print(" ".repeat(padding)))?;
            }

            execute!(stdout, ResetColor)?;
        }

        // Status message
        if let Some(ref msg) = self.status_message {
            let status_row = terminal_height - 2;
            execute!(
                stdout,
                MoveTo(0, status_row),
                SetForegroundColor(Color::Yellow),
                Print(format!(" {} ", msg)),
                ResetColor
            )?;
        }

        // Draw footer with controls
        let footer_row = terminal_height - 1;
        let controls = if self.is_root {
            match self.mode {
                NavigatorMode::Browse => {
                    " ↑↓:Navigate  →/Enter:Open  ←:Up  s:Select  p:Pattern  c:Chmod  q:Quit"
                }
                NavigatorMode::Select => {
                    " ↑↓:Navigate  Space:Toggle  Enter:Confirm  c:Chmod  Esc:Cancel"
                }
                NavigatorMode::PatternSelect => {
                    " Type pattern: 'ali*' for prefix, '.*log' for suffix, '^ali' for exact prefix | Enter:Apply | Esc:Cancel"
                }
                _ => ""
            }
        } else {
            " ↑↓:Navigate  →/Enter:Open  ←/Backspace:Up  Esc/q:Quit"
        };

        execute!(
            stdout,
            MoveTo(0, footer_row),
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
            Print(controls),
            Print(" ".repeat(terminal_width as usize - controls.len())),
            ResetColor
        )?;

        stdout.flush()?;
        Ok(())
    }
}

fn run_app() -> Result<()> {
    // Setup terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    // Initialize navigator
    let mut nav = Navigator::new()?;

    // Main loop
    loop {
        // Update terminal height in case of resize
        nav.terminal_height = terminal::size()?.1;

        // Render
        nav.render()?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            }) = event::read()?
            {
                // Clear status message on any key press
                nav.status_message = None;

                match nav.mode {
                    NavigatorMode::Browse => match code {
                        KeyCode::Up => nav.move_selection_up(),
                        KeyCode::Down => nav.move_selection_down(),
                        KeyCode::Right | KeyCode::Enter => nav.navigate_to_selected()?,
                        KeyCode::Left | KeyCode::Backspace => nav.navigate_up()?,
                        KeyCode::Char('s') if nav.is_root => {
                            nav.mode = NavigatorMode::Select;
                        }
                        KeyCode::Char('p') if nav.is_root => {
                            nav.mode = NavigatorMode::PatternSelect;
                            nav.pattern_input.clear();
                        }
                        KeyCode::Char('c') if nav.is_root => {
                            nav.open_chmod_interface();
                        }
                        KeyCode::Esc | KeyCode::Char('q') => break,
                        _ => {}
                    },
                    NavigatorMode::Select => {
                        match code {
                            KeyCode::Up => nav.move_selection_up(),
                            KeyCode::Down => nav.move_selection_down(),
                            KeyCode::Char(' ') => nav.toggle_selection(),
                            KeyCode::Enter => {
                                if !nav.selected_items.is_empty() {
                                    nav.status_message = Some(format!(
                                        "{} items selected",
                                        nav.selected_items.len()
                                    ));
                                }
                                // Keep in select mode for further operations
                            }
                            KeyCode::Char('c') => {
                                nav.open_chmod_interface();
                            }
                            KeyCode::Esc => {
                                nav.mode = NavigatorMode::Browse;
                                nav.selected_items.clear();
                            }
                            _ => {}
                        }
                    }
                    NavigatorMode::PatternSelect => match code {
                        KeyCode::Enter => {
                            nav.select_by_pattern();
                            nav.mode = NavigatorMode::Select;
                        }
                        KeyCode::Esc => {
                            nav.mode = NavigatorMode::Browse;
                            nav.pattern_input.clear();
                        }
                        KeyCode::Backspace => {
                            nav.pattern_input.pop();
                        }
                        KeyCode::Char(c) => {
                            nav.pattern_input.push(c);
                        }
                        _ => {}
                    },
                    NavigatorMode::ChmodInterface => {
                        if let Some(ref mut chmod) = nav.chmod_interface {
                            if !chmod.handle_input(code) {
                                nav.mode = NavigatorMode::Browse;
                                nav.chmod_interface = None;
                                nav.selected_items.clear();
                                // Reload to show updated permissions
                                let current_dir = nav.current_dir.clone();
                                nav.load_directory(&current_dir)?;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(windows)]
fn main() {
    eprintln!("❌ fsnav does not support Windows directly. Please use WSL.");
    std::process::exit(1);
}

fn main() -> Result<()> {
    let result = run_app();

    // Cleanup terminal
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen, Show)?;
    terminal::disable_raw_mode()?;

    result
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
        };
        assert_eq!(entry.permissions_string(), "rwxr-xr-x");

        let entry2 = FileEntry {
            name: "test2".to_string(),
            path: PathBuf::from("/test2"),
            is_dir: false,
            is_accessible: true,
            is_symlink: false,
            permissions: Some(0o644),
            owner: None,
            group: None,
        };
        assert_eq!(entry2.permissions_string(), "rw-r--r--");
    }
}
