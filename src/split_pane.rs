use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::{
    collections::HashSet,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::models::FileEntry;
use crate::utils::get_owner_group;

#[derive(Debug, Clone, PartialEq)]
pub enum PaneFocus {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub struct Pane {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub selected_items: HashSet<usize>,
    pub scroll_offset: usize,
}

impl Pane {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut pane = Self {
            current_dir: path.clone(),
            entries: Vec::new(),
            selected_index: 0,
            selected_items: HashSet::new(),
            scroll_offset: 0,
        };
        pane.load_directory(&path)?;
        Ok(pane)
    }

    pub fn load_directory(&mut self, path: &Path) -> Result<()> {
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
                    uid: None,
                    gid: None,
                });
            }
        }

        // Read directory entries
        match std::fs::read_dir(path) {
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

                    let permissions = metadata.as_ref().ok().map(|m| {
                        use std::os::unix::fs::PermissionsExt;
                        m.permissions().mode()
                    });

                    let (owner, group, uid, gid) = get_owner_group(&path);

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
                        uid,
                        gid,
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
                self.entries.push(FileEntry {
                    name: format!("⚠️  Error: {}", e),
                    path: path.to_path_buf(),
                    is_dir: false,
                    is_accessible: false,
                    is_symlink: false,
                    permissions: None,
                    owner: None,
                    group: None,
                    uid: None,
                    gid: None,
                });
            }
        }

        self.current_dir = path.to_path_buf();
        Ok(())
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
            self.adjust_scroll();
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index < self.entries.len().saturating_sub(1) {
            self.selected_index += 1;
            self.adjust_scroll();
        }
    }

    pub fn navigate_to_selected(&mut self) -> Result<()> {
        if let Some(entry) = self.entries.get(self.selected_index) {
            if entry.is_dir && entry.is_accessible {
                let new_path = entry.path.clone();
                self.load_directory(&new_path)?;
            }
        }
        Ok(())
    }

    pub fn navigate_up(&mut self) -> Result<()> {
        if let Some(parent) = self.current_dir.parent() {
            let parent_path = parent.to_path_buf();
            self.load_directory(&parent_path)?;
        }
        Ok(())
    }

    pub fn toggle_selection(&mut self) {
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

    pub fn get_selected_paths(&self) -> Vec<PathBuf> {
        if self.selected_items.is_empty() {
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
            self.selected_items
                .iter()
                .filter_map(|&i| self.entries.get(i))
                .filter(|e| e.name != "..")
                .map(|e| e.path.clone())
                .collect()
        }
    }

    fn adjust_scroll(&mut self) {
        // This will be calculated based on available height
    }

    pub fn adjust_scroll_with_height(&mut self, visible_height: usize) {
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index.saturating_sub(visible_height - 1);
        }
    }
}

pub struct SplitPaneView {
    pub left_pane: Pane,
    pub right_pane: Pane,
    pub focus: PaneFocus,
    pub vertical_split: bool,
    pub split_ratio: f32, // 0.0 to 1.0, percentage for left/top pane
}

impl SplitPaneView {
    pub fn new(left_path: PathBuf, right_path: PathBuf) -> Result<Self> {
        Ok(Self {
            left_pane: Pane::new(left_path)?,
            right_pane: Pane::new(right_path)?,
            focus: PaneFocus::Left,
            vertical_split: true,
            split_ratio: 0.5,
        })
    }

    pub fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            PaneFocus::Left => PaneFocus::Right,
            PaneFocus::Right => PaneFocus::Left,
        };
    }

    pub fn toggle_layout(&mut self) {
        self.vertical_split = !self.vertical_split;
    }

    pub fn adjust_split(&mut self, delta: f32) {
        self.split_ratio = (self.split_ratio + delta).clamp(0.2, 0.8);
    }

    pub fn get_active_pane(&self) -> &Pane {
        match self.focus {
            PaneFocus::Left => &self.left_pane,
            PaneFocus::Right => &self.right_pane,
        }
    }

    pub fn get_active_pane_mut(&mut self) -> &mut Pane {
        match self.focus {
            PaneFocus::Left => &mut self.left_pane,
            PaneFocus::Right => &mut self.right_pane,
        }
    }

    pub fn sync_directories(&mut self) -> Result<()> {
        let target_dir = self.get_active_pane().current_dir.clone();
        match self.focus {
            PaneFocus::Left => self.right_pane.load_directory(&target_dir)?,
            PaneFocus::Right => self.left_pane.load_directory(&target_dir)?,
        }
        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        let mut stdout = io::stdout();
        let (terminal_width, terminal_height) = terminal::size()?;

        // Clear screen
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

        if self.vertical_split {
            self.render_vertical_split(&mut stdout, terminal_width, terminal_height)?;
        } else {
            self.render_horizontal_split(&mut stdout, terminal_width, terminal_height)?;
        }

        // Render status bar
        self.render_status_bar(&mut stdout, terminal_width, terminal_height)?;

        stdout.flush()?;
        Ok(())
    }

    fn render_vertical_split(
        &mut self,
        stdout: &mut io::Stdout,
        width: u16,
        height: u16,
    ) -> Result<()> {
        let split_pos = (width as f32 * self.split_ratio) as u16;
        let left_width = split_pos.saturating_sub(1);
        let right_width = width.saturating_sub(split_pos + 1);

        // Render left pane
        Self::render_pane(
            stdout,
            &mut self.left_pane,
            0,
            0,
            left_width,
            height - 2,
            self.focus == PaneFocus::Left,
        )?;

        // Render divider
        for y in 0..height - 2 {
            execute!(
            stdout,
            MoveTo(split_pos, y),
            SetForegroundColor(Color::DarkGrey),
            Print("│"),
            ResetColor
        )?;
        }

        // Render right pane
        Self::render_pane(
            stdout,
            &mut self.right_pane,
            split_pos + 1,
            0,
            right_width,
            height - 2,
            self.focus == PaneFocus::Right,
        )?;

        Ok(())
    }

    fn render_horizontal_split(
        &mut self,
        stdout: &mut io::Stdout,
        width: u16,
        height: u16,
    ) -> Result<()> {
        let split_pos = ((height - 2) as f32 * self.split_ratio) as u16;
        let top_height = split_pos;
        let bottom_height = (height - 2).saturating_sub(split_pos + 1);

        // Render top pane
        Self::render_pane(
            stdout,
            &mut self.left_pane,
            0,
            0,
            width,
            top_height,
            self.focus == PaneFocus::Left,
        )?;

        // Render divider
        execute!(
        stdout,
        MoveTo(0, split_pos),
        SetForegroundColor(Color::DarkGrey),
        Print("─".repeat(width as usize)),
        ResetColor
    )?;

        // Render bottom pane
        Self::render_pane(
            stdout,
            &mut self.right_pane,
            0,
            split_pos + 1,
            width,
            bottom_height,
            self.focus == PaneFocus::Right,
        )?;

        Ok(())
    }

    fn render_pane(
        stdout: &mut io::Stdout,
        pane: &mut Pane,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        is_active: bool,
    ) -> Result<()> {
        // Header
        let header_color = if is_active {
            Color::Blue
        } else {
            Color::DarkGrey
        };

        execute!(
            stdout,
            MoveTo(x, y),
            SetBackgroundColor(header_color),
            SetForegroundColor(Color::White),
            Print(format!(
                " {} ",
                pane.current_dir.to_string_lossy()
                    .chars()
                    .take((width - 2) as usize)
                    .collect::<String>()
            )),
            Print(" ".repeat((width as usize).saturating_sub(
                pane.current_dir.to_string_lossy().len() + 2
            ))),
            ResetColor
        )?;

        // File list
        let list_height = (height - 1) as usize;
        pane.adjust_scroll_with_height(list_height);

        let end_index = (pane.scroll_offset + list_height).min(pane.entries.len());

        for (i, entry) in pane.entries[pane.scroll_offset..end_index].iter().enumerate() {
            let row = y + 1 + i as u16;
            let display_index = pane.scroll_offset + i;
            let is_selected = pane.selected_items.contains(&display_index);
            let is_highlighted = display_index == pane.selected_index;

            execute!(stdout, MoveTo(x, row))?;

            if is_highlighted && is_active {
                execute!(
                    stdout,
                    SetBackgroundColor(Color::DarkGreen),
                    SetForegroundColor(Color::White)
                )?;
            } else if is_highlighted {
                execute!(
                    stdout,
                    SetBackgroundColor(Color::DarkGrey),
                    SetForegroundColor(Color::White)
                )?;
            }

            let marker = if is_selected { "[✓]" } else { "   " };
            let prefix = if is_highlighted { ">" } else { " " };

            let display_name = entry.display_name();
            let truncated_name = if display_name.len() > (width - 5) as usize {
                format!("{}...", &display_name[..(width - 8) as usize])
            } else {
                display_name
            };

            execute!(
                stdout,
                Print(format!("{}{} {}", prefix, marker, truncated_name))
            )?;

            if is_highlighted {
                let padding = (width as usize).saturating_sub(
                    prefix.len() + marker.len() + truncated_name.len() + 1
                );
                execute!(stdout, Print(" ".repeat(padding)))?;
            }

            execute!(stdout, ResetColor)?;
        }

        Ok(())
    }

    fn render_status_bar(
        &self,
        stdout: &mut io::Stdout,
        width: u16,
        height: u16,
    ) -> Result<()> {
        let status = format!(
            " Tab: Switch Pane | F5: Sync Dirs | F6: Toggle Layout | +/-: Adjust Split | q: Quit"
        );

        execute!(
            stdout,
            MoveTo(0, height - 1),
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
            Print(&status),
            Print(" ".repeat((width as usize).saturating_sub(status.len()))),
            ResetColor
        )?;

        Ok(())
    }
}