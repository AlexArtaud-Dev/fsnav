use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    event::KeyCode,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::{
    io::{self, Write},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct ChownInterface {
    // Selected files/directories
    selected_paths: Vec<PathBuf>,
    // Available users and groups
    users: Vec<UserInfo>,
    groups: Vec<GroupInfo>,
    // Current selected user and group indices (in filtered list)
    selected_user_idx: usize,
    selected_group_idx: usize,
    // Search/filter strings
    user_search: String,
    group_search: String,
    // UI state
    focus: Focus,
    show_preview: bool,
    recursive: bool,
    // Changes history
    history: Vec<OwnershipChange>,
    // Warnings for critical files
    warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
enum Focus {
    UserList,
    GroupList,
    Options,
    Confirm,
}

#[derive(Debug, Clone)]
struct UserInfo {
    uid: u32,
    name: String,
    full_name: Option<String>,
}

#[derive(Debug, Clone)]
struct GroupInfo {
    gid: u32,
    name: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct OwnershipChange {
    path: PathBuf,
    old_uid: u32,
    old_gid: u32,
    new_uid: u32,
    new_gid: u32,
    timestamp: std::time::SystemTime,
}

impl ChownInterface {
    pub fn new(selected_paths: Vec<PathBuf>) -> Self {
        let users = Self::get_system_users();
        let groups = Self::get_system_groups();
        let warnings = Self::check_critical_paths(&selected_paths);

        // Try to find current user/group from first file
        let (current_uid, current_gid) = if let Some(first_path) = selected_paths.first() {
            Self::get_file_ownership(first_path)
        } else {
            (0, 0)
        };

        let selected_user_idx = users.iter().position(|u| u.uid == current_uid).unwrap_or(0);

        let selected_group_idx = groups
            .iter()
            .position(|g| g.gid == current_gid)
            .unwrap_or(0);

        Self {
            selected_paths,
            users,
            groups,
            selected_user_idx,
            selected_group_idx,
            user_search: String::new(),
            group_search: String::new(),
            focus: Focus::UserList,
            show_preview: true,
            recursive: false,
            history: Vec::new(),
            warnings,
        }
    }

    fn get_system_users() -> Vec<UserInfo> {
        let mut users = Vec::new();

        #[cfg(unix)]
        {
            use std::fs::File;
            use std::io::{BufRead, BufReader};

            if let Ok(file) = File::open("/etc/passwd") {
                let reader = BufReader::new(file);
                for line in reader.lines().map_while(Result::ok) {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 5 {
                        if let Ok(uid) = parts[2].parse::<u32>() {
                            users.push(UserInfo {
                                uid,
                                name: parts[0].to_string(),
                                full_name: if parts[4].is_empty() {
                                    None
                                } else {
                                    Some(parts[4].split(',').next().unwrap_or("").to_string())
                                },
                            });
                        }
                    }
                }
            }
        }

        users.sort_by_key(|u: &UserInfo| u.name.clone());
        users
    }

    fn get_system_groups() -> Vec<GroupInfo> {
        let mut groups = Vec::new();

        #[cfg(unix)]
        {
            use std::fs::File;
            use std::io::{BufRead, BufReader};

            if let Ok(file) = File::open("/etc/group") {
                let reader = BufReader::new(file);
                for line in reader.lines().map_while(Result::ok) {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 3 {
                        if let Ok(gid) = parts[2].parse::<u32>() {
                            groups.push(GroupInfo {
                                gid,
                                name: parts[0].to_string(),
                            });
                        }
                    }
                }
            }
        }

        groups.sort_by_key(|g: &GroupInfo| g.name.clone());
        groups
    }

    fn get_file_ownership(_path: &PathBuf) -> (u32, u32) {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(metadata) = _path.metadata() {
                return (metadata.uid(), metadata.gid());
            }
        }
        (0, 0)
    }

    fn check_critical_paths(paths: &[PathBuf]) -> Vec<String> {
        let mut warnings = Vec::new();
        let critical_paths = [
            "/etc",
            "/bin",
            "/sbin",
            "/usr/bin",
            "/usr/sbin",
            "/boot",
            "/lib",
            "/lib64",
            "/proc",
            "/sys",
            "/dev",
        ];

        for path in paths {
            let path_str = path.to_string_lossy();
            for critical in &critical_paths {
                if path_str.starts_with(critical) {
                    warnings.push(format!(
                        "⚠️ {} is in a critical system directory!",
                        path.display()
                    ));
                }
            }
        }

        warnings
    }

    pub fn render(&self) -> Result<()> {
        let mut stdout = io::stdout();
        let (terminal_width, terminal_height) = terminal::size()?;

        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

        // Title
        self.render_title(&mut stdout)?;

        // Warnings if any
        if !self.warnings.is_empty() {
            self.render_warnings(&mut stdout, 4)?;
        }

        let content_start = if self.warnings.is_empty() {
            4
        } else {
            4 + self.warnings.len() as u16 + 1
        };

        // Main content area
        self.render_main_content(&mut stdout, content_start, terminal_width)?;

        // Preview if enabled
        if self.show_preview {
            self.render_preview(&mut stdout, content_start + 14, terminal_width)?;
            // Adjusted for 5 items
        }

        // Controls
        self.render_controls(&mut stdout, terminal_height - 2)?;

        stdout.flush()?;
        Ok(())
    }

    fn render_title(&self, stdout: &mut io::Stdout) -> Result<()> {
        execute!(
            stdout,
            MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("╔══════════════════════════════════════════════════════════════════════╗"),
            MoveTo(0, 1),
            Print("║           INTERACTIVE CHOWN - Ownership Manager                      ║"),
            MoveTo(0, 2),
            Print("╚══════════════════════════════════════════════════════════════════════╝"),
            ResetColor
        )?;
        Ok(())
    }

    fn render_warnings(&self, stdout: &mut io::Stdout, y: u16) -> Result<()> {
        for (i, warning) in self.warnings.iter().enumerate() {
            execute!(
                stdout,
                MoveTo(0, y + i as u16),
                SetBackgroundColor(Color::DarkRed),
                SetForegroundColor(Color::White),
                Print(format!(" {} ", warning)),
                ResetColor
            )?;
        }
        Ok(())
    }

    fn render_main_content(&self, stdout: &mut io::Stdout, y: u16, width: u16) -> Result<()> {
        // Selected files info
        execute!(
            stdout,
            MoveTo(2, y),
            SetForegroundColor(Color::Yellow),
            Print(format!(
                "📁 Selected: {} item(s)",
                self.selected_paths.len()
            )),
            ResetColor
        )?;

        // User selection area
        let user_area_y = y + 2;
        execute!(
            stdout,
            MoveTo(2, user_area_y),
            SetForegroundColor(if self.focus == Focus::UserList {
                Color::Green
            } else {
                Color::Cyan
            }),
            Print("👤 USER SELECTION"),
            ResetColor
        )?;

        // User search box
        execute!(
            stdout,
            MoveTo(4, user_area_y + 1),
            Print("Search: "),
            SetForegroundColor(Color::White),
            Print(if self.focus == Focus::UserList {
                format!("{}_", self.user_search)
            } else {
                self.user_search.clone()
            }),
            ResetColor
        )?;

        // Filtered users list (show 5 items)
        let filtered_users: Vec<&UserInfo> = self
            .users
            .iter()
            .filter(|u| {
                self.user_search.is_empty()
                    || u.name
                        .to_lowercase()
                        .contains(&self.user_search.to_lowercase())
            })
            .collect();

        if !filtered_users.is_empty() {
            // Ensure selected index is within bounds of filtered list
            let safe_selected_idx = self.selected_user_idx.min(filtered_users.len() - 1);

            // Calculate start index for display window
            let display_count = 5.min(filtered_users.len());
            let start_idx = if safe_selected_idx >= display_count - 1 {
                safe_selected_idx.saturating_sub(display_count - 1)
            } else {
                0
            };

            for i in 0..display_count {
                let idx = start_idx + i;
                if let Some(user) = filtered_users.get(idx) {
                    let is_selected = idx == safe_selected_idx && self.focus == Focus::UserList;
                    execute!(
                        stdout,
                        MoveTo(4, user_area_y + 2 + i as u16),
                        if is_selected {
                            SetBackgroundColor(Color::DarkGreen)
                        } else {
                            SetBackgroundColor(Color::Black)
                        },
                        SetForegroundColor(if is_selected {
                            Color::White
                        } else {
                            Color::Grey
                        }),
                        Print(format!(
                            " {} {:<12} ({:>5}) {:<20} ",
                            if is_selected { ">" } else { " " },
                            &user.name[..user.name.len().min(12)],
                            user.uid,
                            user.full_name
                                .as_ref()
                                .map(|s| &s[..s.len().min(20)])
                                .unwrap_or("")
                        )),
                        ResetColor
                    )?;
                }
            }
        }

        // Group selection area
        let group_x = width / 2;

        execute!(
            stdout,
            MoveTo(group_x, user_area_y),
            SetForegroundColor(if self.focus == Focus::GroupList {
                Color::Green
            } else {
                Color::Cyan
            }),
            Print("👥 GROUP SELECTION"),
            ResetColor
        )?;

        // Group search box
        execute!(
            stdout,
            MoveTo(group_x + 2, user_area_y + 1),
            Print("Search: "),
            SetForegroundColor(Color::White),
            Print(if self.focus == Focus::GroupList {
                format!("{}_", self.group_search)
            } else {
                self.group_search.clone()
            }),
            ResetColor
        )?;

        // Filtered groups list (show 5 items)
        let filtered_groups: Vec<&GroupInfo> = self
            .groups
            .iter()
            .filter(|g| {
                self.group_search.is_empty()
                    || g.name
                        .to_lowercase()
                        .contains(&self.group_search.to_lowercase())
            })
            .collect();

        if !filtered_groups.is_empty() {
            // Ensure selected index is within bounds of filtered list
            let safe_selected_idx = self.selected_group_idx.min(filtered_groups.len() - 1);

            // Calculate start index for display window
            let display_count = 5.min(filtered_groups.len());
            let start_idx = if safe_selected_idx >= display_count - 1 {
                safe_selected_idx.saturating_sub(display_count - 1)
            } else {
                0
            };

            for i in 0..display_count {
                let idx = start_idx + i;
                if let Some(group) = filtered_groups.get(idx) {
                    let is_selected = idx == safe_selected_idx && self.focus == Focus::GroupList;
                    execute!(
                        stdout,
                        MoveTo(group_x + 2, user_area_y + 2 + i as u16),
                        if is_selected {
                            SetBackgroundColor(Color::DarkGreen)
                        } else {
                            SetBackgroundColor(Color::Black)
                        },
                        SetForegroundColor(if is_selected {
                            Color::White
                        } else {
                            Color::Grey
                        }),
                        Print(format!(
                            " {} {:<15} ({:>5}) ",
                            if is_selected { ">" } else { " " },
                            &group.name[..group.name.len().min(15)],
                            group.gid
                        )),
                        ResetColor
                    )?;
                }
            }
        }

        // Options area
        let options_y = user_area_y + 8; // Adjusted for 5 items instead of 3
        execute!(
            stdout,
            MoveTo(2, options_y),
            SetForegroundColor(if self.focus == Focus::Options {
                Color::Green
            } else {
                Color::Cyan
            }),
            Print("⚙️ OPTIONS"),
            ResetColor
        )?;

        execute!(
            stdout,
            MoveTo(4, options_y + 1),
            if self.recursive {
                SetForegroundColor(Color::Green)
            } else {
                SetForegroundColor(Color::DarkGrey)
            },
            Print(format!(
                "[{}] Recursive (-R) - Apply to all subdirectories and files",
                if self.recursive { "✓" } else { " " }
            )),
            ResetColor
        )?;

        Ok(())
    }

    fn render_preview(&self, stdout: &mut io::Stdout, y: u16, _width: u16) -> Result<()> {
        execute!(
            stdout,
            MoveTo(2, y),
            SetForegroundColor(Color::Yellow),
            Print("📊 PREVIEW - Files to be affected:"),
            ResetColor
        )?;

        // Get filtered lists to show correct preview
        let filtered_users: Vec<&UserInfo> = self
            .users
            .iter()
            .filter(|u| {
                self.user_search.is_empty()
                    || u.name
                        .to_lowercase()
                        .contains(&self.user_search.to_lowercase())
            })
            .collect();

        let filtered_groups: Vec<&GroupInfo> = self
            .groups
            .iter()
            .filter(|g| {
                self.group_search.is_empty()
                    || g.name
                        .to_lowercase()
                        .contains(&self.group_search.to_lowercase())
            })
            .collect();

        let selected_user = filtered_users.get(
            self.selected_user_idx
                .min(filtered_users.len().saturating_sub(1)),
        );
        let selected_group = filtered_groups.get(
            self.selected_group_idx
                .min(filtered_groups.len().saturating_sub(1)),
        );

        // Show affected files
        let mut all_files = Vec::new();
        for path in &self.selected_paths {
            all_files.push(path.clone());
            if self.recursive && path.is_dir() {
                // In real implementation, would recursively get all files
                // For now, just show indication
                all_files.push(PathBuf::from(format!(
                    "  {} (and all contents)",
                    path.display()
                )));
            }
        }

        for (i, file) in all_files.iter().take(5).enumerate() {
            let (current_uid, current_gid) = Self::get_file_ownership(file);
            let current_user = self.users.iter().find(|u| u.uid == current_uid);
            let current_group = self.groups.iter().find(|g| g.gid == current_gid);

            execute!(
                stdout,
                MoveTo(4, y + 1 + i as u16),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("• {}", file.display())),
                ResetColor
            )?;

            execute!(
                stdout,
                MoveTo(6, y + 2 + i as u16),
                SetForegroundColor(Color::Red),
                Print(format!(
                    "  {} : {} ",
                    current_user.map(|u| u.name.as_str()).unwrap_or("?"),
                    current_group.map(|g| g.name.as_str()).unwrap_or("?")
                )),
                SetForegroundColor(Color::White),
                Print("→"),
                SetForegroundColor(Color::Green),
                Print(format!(
                    " {} : {}",
                    selected_user.map(|u| u.name.as_str()).unwrap_or("?"),
                    selected_group.map(|g| g.name.as_str()).unwrap_or("?")
                )),
                ResetColor
            )?;
        }

        if all_files.len() > 5 {
            execute!(
                stdout,
                MoveTo(4, y + 6),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("... and {} more files", all_files.len() - 5)),
                ResetColor
            )?;
        }

        Ok(())
    }

    fn render_controls(&self, stdout: &mut io::Stdout, y: u16) -> Result<()> {
        let controls = match self.focus {
            Focus::UserList | Focus::GroupList => {
                " Tab: Switch Focus | ↑↓: Navigate | Type: Search | r: Toggle Recursive | p: Toggle Preview | Enter: Apply | Esc: Cancel "
            }
            Focus::Options => {
                " Tab: Switch Focus | Space/r: Toggle Recursive | p: Toggle Preview | Enter: Apply | Esc: Cancel "
            }
            Focus::Confirm => {
                " y: Yes, Apply Changes | n/Esc: No, Cancel "
            }
        };

        execute!(
            stdout,
            MoveTo(0, y),
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
            Print(controls),
            ResetColor
        )?;

        Ok(())
    }

    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Tab => {
                self.focus = match self.focus {
                    Focus::UserList => Focus::GroupList,
                    Focus::GroupList => Focus::Options,
                    Focus::Options => Focus::UserList,
                    Focus::Confirm => Focus::Confirm,
                };
            }
            KeyCode::Up => {
                match self.focus {
                    Focus::UserList => {
                        // Filter users first
                        let filtered_users: Vec<&UserInfo> = self
                            .users
                            .iter()
                            .filter(|u| {
                                self.user_search.is_empty()
                                    || u.name
                                        .to_lowercase()
                                        .contains(&self.user_search.to_lowercase())
                            })
                            .collect();

                        if !filtered_users.is_empty() && self.selected_user_idx > 0 {
                            self.selected_user_idx -= 1;
                        }
                    }
                    Focus::GroupList => {
                        // Filter groups first
                        let filtered_groups: Vec<&GroupInfo> = self
                            .groups
                            .iter()
                            .filter(|g| {
                                self.group_search.is_empty()
                                    || g.name
                                        .to_lowercase()
                                        .contains(&self.group_search.to_lowercase())
                            })
                            .collect();

                        if !filtered_groups.is_empty() && self.selected_group_idx > 0 {
                            self.selected_group_idx -= 1;
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Down => {
                match self.focus {
                    Focus::UserList => {
                        // Filter users first
                        let filtered_users: Vec<&UserInfo> = self
                            .users
                            .iter()
                            .filter(|u| {
                                self.user_search.is_empty()
                                    || u.name
                                        .to_lowercase()
                                        .contains(&self.user_search.to_lowercase())
                            })
                            .collect();

                        if !filtered_users.is_empty()
                            && self.selected_user_idx < filtered_users.len() - 1
                        {
                            self.selected_user_idx += 1;
                        }
                    }
                    Focus::GroupList => {
                        // Filter groups first
                        let filtered_groups: Vec<&GroupInfo> = self
                            .groups
                            .iter()
                            .filter(|g| {
                                self.group_search.is_empty()
                                    || g.name
                                        .to_lowercase()
                                        .contains(&self.group_search.to_lowercase())
                            })
                            .collect();

                        if !filtered_groups.is_empty()
                            && self.selected_group_idx < filtered_groups.len() - 1
                        {
                            self.selected_group_idx += 1;
                        }
                    }
                    _ => {}
                }
            }
            KeyCode::Char(' ') if self.focus == Focus::Options => {
                self.recursive = !self.recursive;
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.recursive = !self.recursive;
            }
            KeyCode::Char('p') | KeyCode::Char('P') => {
                self.show_preview = !self.show_preview;
            }
            KeyCode::Backspace => {
                match self.focus {
                    Focus::UserList => {
                        self.user_search.pop();
                        // Reset selection when search changes
                        self.selected_user_idx = 0;
                    }
                    Focus::GroupList => {
                        self.group_search.pop();
                        // Reset selection when search changes
                        self.selected_group_idx = 0;
                    }
                    _ => {}
                }
            }
            KeyCode::Char(c) if c.is_alphanumeric() || c == '_' || c == '-' => {
                match self.focus {
                    Focus::UserList => {
                        self.user_search.push(c);
                        // Reset selection to first item when search changes
                        self.selected_user_idx = 0;
                    }
                    Focus::GroupList => {
                        self.group_search.push(c);
                        // Reset selection to first item when search changes
                        self.selected_group_idx = 0;
                    }
                    _ => {}
                }
            }
            KeyCode::Enter => {
                if !self.warnings.is_empty() && self.focus != Focus::Confirm {
                    self.focus = Focus::Confirm;
                } else {
                    self.apply_ownership_changes();
                    return false; // Exit interface
                }
            }
            KeyCode::Char('y') | KeyCode::Char('Y') if self.focus == Focus::Confirm => {
                self.apply_ownership_changes();
                return false; // Exit interface
            }
            KeyCode::Char('n') | KeyCode::Char('N') if self.focus == Focus::Confirm => {
                return false; // Exit without applying
            }
            KeyCode::Esc => {
                if self.focus == Focus::Confirm {
                    self.focus = Focus::UserList;
                } else {
                    return false; // Exit without applying
                }
            }
            _ => {}
        }
        true // Continue
    }

    fn apply_ownership_changes(&mut self) {
        // Get filtered lists
        let filtered_users: Vec<&UserInfo> = self
            .users
            .iter()
            .filter(|u| {
                self.user_search.is_empty()
                    || u.name
                        .to_lowercase()
                        .contains(&self.user_search.to_lowercase())
            })
            .collect();

        let filtered_groups: Vec<&GroupInfo> = self
            .groups
            .iter()
            .filter(|g| {
                self.group_search.is_empty()
                    || g.name
                        .to_lowercase()
                        .contains(&self.group_search.to_lowercase())
            })
            .collect();

        // Get the actual selected items from filtered lists
        let selected_user = filtered_users.get(
            self.selected_user_idx
                .min(filtered_users.len().saturating_sub(1)),
        );
        let selected_group = filtered_groups.get(
            self.selected_group_idx
                .min(filtered_groups.len().saturating_sub(1)),
        );

        if let (Some(&user), Some(&group)) = (selected_user, selected_group) {
            for path in &self.selected_paths {
                let (old_uid, old_gid) = Self::get_file_ownership(path);

                // Record the change in history
                self.history.push(OwnershipChange {
                    path: path.clone(),
                    old_uid,
                    old_gid,
                    new_uid: user.uid,
                    new_gid: group.gid,
                    timestamp: std::time::SystemTime::now(),
                });

                // Apply the ownership change
                self.change_ownership(path, user.uid, group.gid);

                // If recursive and directory, apply to contents
                if self.recursive && path.is_dir() {
                    self.apply_recursive(path, user.uid, group.gid);
                }
            }
        }
    }

    fn change_ownership(&self, _path: &PathBuf, _uid: u32, _gid: u32) {
        #[cfg(unix)]
        {
            use std::os::unix::fs;
            let _ = fs::chown(_path, Some(_uid), Some(_gid));
        }
    }

    fn apply_recursive(&self, _dir: &PathBuf, _uid: u32, _gid: u32) {
        #[cfg(unix)]
        {
            use std::fs;
            if let Ok(entries) = fs::read_dir(_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    self.change_ownership(&path, _uid, _gid);
                    if path.is_dir() {
                        self.apply_recursive(&path, _uid, _gid);
                    }
                }
            }
        }
    }
}
