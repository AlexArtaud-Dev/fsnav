use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal,
};
use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

use crate::managers::{ChmodInterface, ChownInterface};
use crate::models::{ExitAction, FileEntry};
use crate::ui::{RenderContext, Renderer};
use crate::utils::{get_owner_group, is_root_user, match_pattern};

#[derive(Debug, PartialEq)]
pub enum NavigatorMode {
    Browse,
    Select,
    ChmodInterface,
    ChownInterface,
    PatternSelect,
}

pub struct Navigator {
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
    chown_interface: Option<ChownInterface>,
    status_message: Option<String>,
    renderer: Renderer,
}

impl Navigator {
    pub fn new() -> Result<Self> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        let is_root = is_root_user();

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
            chown_interface: None,
            status_message: None,
            renderer: Renderer::new(),
        };
        nav.load_directory(&current_dir)?;
        Ok(nav)
    }

    #[allow(dead_code)]
    pub fn get_current_dir(&self) -> &Path {
        &self.current_dir
    }

    pub fn run(&mut self) -> Result<ExitAction> {
        loop {
            // Update terminal height in case of resize
            self.terminal_height = terminal::size()?.1;

            // Render
            self.render()?;

            // Handle input
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind: KeyEventKind::Press,
                    ..
                }) = event::read()?
                {
                    if let Some(action) = self.handle_input(code, modifiers)? {
                        return Ok(action);
                    }
                }
            }
        }
    }

    fn render(&self) -> Result<()> {
        match self.mode {
            NavigatorMode::ChmodInterface => {
                if let Some(ref chmod) = self.chmod_interface {
                    return chmod.render();
                }
            }
            NavigatorMode::ChownInterface => {
                if let Some(ref chown) = self.chown_interface {
                    return chown.render();
                }
            }
            _ => {}
        }

        let ctx = RenderContext {
            current_dir: &self.current_dir,
            entries: &self.entries,
            selected_index: self.selected_index,
            selected_items: &self.selected_items,
            scroll_offset: self.scroll_offset,
            terminal_height: self.terminal_height,
            mode: &self.mode,
            is_root: self.is_root,
            pattern_input: &self.pattern_input,
            status_message: &self.status_message,
        };

        self.renderer.render(ctx)
    }

    fn handle_input(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Result<Option<ExitAction>> {
        // Clear status message on any key press
        self.status_message = None;

        match self.mode {
            NavigatorMode::Browse => match code {
                KeyCode::Up => self.move_selection_up(),
                KeyCode::Down => self.move_selection_down(),
                KeyCode::Right | KeyCode::Enter => self.navigate_to_selected()?,
                KeyCode::Left | KeyCode::Backspace => self.navigate_up()?,
                KeyCode::Char('s') if self.is_root => {
                    self.mode = NavigatorMode::Select;
                }
                KeyCode::Char('p') if self.is_root => {
                    self.mode = NavigatorMode::PatternSelect;
                    self.pattern_input.clear();
                }
                KeyCode::Char('c') if self.is_root => {
                    self.open_chmod_interface();
                }
                KeyCode::Char('o') if self.is_root => {
                    self.open_chown_interface();
                }
                // Ctrl+D to spawn shell in current directory
                KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(Some(ExitAction::SpawnShell(self.current_dir.clone())));
                }
                // Shift+S to spawn shell
                KeyCode::Char('S') => {
                    return Ok(Some(ExitAction::SpawnShell(self.current_dir.clone())));
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    return Ok(Some(ExitAction::Quit));
                }
                _ => {}
            },
            NavigatorMode::Select => match code {
                KeyCode::Up => self.move_selection_up(),
                KeyCode::Down => self.move_selection_down(),
                KeyCode::Char(' ') => self.toggle_selection(),
                KeyCode::Enter => {
                    if !self.selected_items.is_empty() {
                        self.status_message =
                            Some(format!("{} items selected", self.selected_items.len()));
                    }
                }
                KeyCode::Char('c') => {
                    self.open_chmod_interface();
                }
                KeyCode::Char('o') => {
                    self.open_chown_interface();
                }
                KeyCode::Esc => {
                    self.mode = NavigatorMode::Browse;
                    self.selected_items.clear();
                }
                _ => {}
            },
            NavigatorMode::PatternSelect => match code {
                KeyCode::Enter => {
                    self.select_by_pattern();
                    self.mode = NavigatorMode::Select;
                }
                KeyCode::Esc => {
                    self.mode = NavigatorMode::Browse;
                    self.pattern_input.clear();
                }
                KeyCode::Backspace => {
                    self.pattern_input.pop();
                }
                KeyCode::Char(c) => {
                    self.pattern_input.push(c);
                }
                _ => {}
            },
            NavigatorMode::ChmodInterface => {
                if let Some(ref mut chmod) = self.chmod_interface {
                    if !chmod.handle_input(code) {
                        self.mode = NavigatorMode::Browse;
                        self.chmod_interface = None;
                        self.selected_items.clear();
                        // Reload to show updated permissions
                        let current_dir = self.current_dir.clone();
                        self.load_directory(&current_dir)?;
                    }
                }
            }
            NavigatorMode::ChownInterface => {
                if let Some(ref mut chown) = self.chown_interface {
                    if !chown.handle_input(code) {
                        self.mode = NavigatorMode::Browse;
                        self.chown_interface = None;
                        self.selected_items.clear();
                        // Reload to show updated ownership
                        let current_dir = self.current_dir.clone();
                        self.load_directory(&current_dir)?;
                    }
                }
            }
        }
        Ok(None)
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
                    uid: None,
                    gid: None,
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

                    let permissions = metadata.as_ref().ok().map(|m| {
                        use std::os::unix::fs::PermissionsExt;
                        m.permissions().mode()
                    });

                    // Get owner and group info
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
                    uid: None,
                    gid: None,
                });
            }
        }

        self.current_dir = path.to_path_buf();
        Ok(())
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

        for (i, entry) in self.entries.iter().enumerate() {
            if entry.name != ".." && match_pattern(&self.pattern_input, &entry.name) {
                self.selected_items.insert(i);
            }
        }

        self.status_message = Some(format!(
            "Selected {} items matching '{}'",
            self.selected_items.len(),
            self.pattern_input
        ));

        self.pattern_input.clear();
    }

    fn open_chmod_interface(&mut self) {
        if !self.is_root {
            self.status_message = Some("⚠️  Chmod interface requires root privileges".to_string());
            return;
        }

        let selected_paths = self.get_selected_paths();
        if selected_paths.is_empty() {
            self.status_message = Some("No items selected for chmod".to_string());
            return;
        }

        self.chmod_interface = Some(ChmodInterface::new(selected_paths));
        self.mode = NavigatorMode::ChmodInterface;
    }

    fn open_chown_interface(&mut self) {
        if !self.is_root {
            self.status_message = Some("⚠️  Chown interface requires root privileges".to_string());
            return;
        }

        let selected_paths = self.get_selected_paths();
        if selected_paths.is_empty() {
            self.status_message = Some("No items selected for chown".to_string());
            return;
        }

        self.chown_interface = Some(ChownInterface::new(selected_paths));
        self.mode = NavigatorMode::ChownInterface;
    }

    fn get_selected_paths(&self) -> Vec<PathBuf> {
        if self.selected_items.is_empty() {
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
        }
    }

    fn adjust_scroll(&mut self) {
        let visible_area = (self.terminal_height as usize).saturating_sub(5);

        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_area {
            self.scroll_offset = self.selected_index.saturating_sub(visible_area - 1);
        }
    }
}
