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

use crate::bookmarks::BookmarksManager;
use crate::managers::{ChmodInterface, ChownInterface};
use crate::models::{ExitAction, FileEntry};
use crate::preview::FilePreview;
use crate::search::SearchMode;
use crate::split_pane::SplitPaneView;
use crate::ui::{RenderContext, Renderer};
use crate::utils::{get_owner_group, is_root_user, match_pattern};

#[derive(Debug, PartialEq)]
pub enum NavigatorMode {
    Browse,
    Select,
    ChmodInterface,
    ChownInterface,
    PatternSelect,
    Search,
    Preview,
    Bookmarks,
    SplitPane,
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
    // New v0.4.0 features
    search_mode: Option<SearchMode>,
    file_preview: Option<FilePreview>,
    bookmarks_manager: BookmarksManager,
    split_pane_view: Option<SplitPaneView>,
    show_preview_panel: bool,
    // Add these new fields for fixes
    bookmark_selected_index: Option<usize>,
    preview_focused: bool,
}

impl Navigator {
    pub fn new() -> Result<Self> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        let is_root = is_root_user();
        let bookmarks_manager = BookmarksManager::new()?;

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
            search_mode: None,
            file_preview: None,
            bookmarks_manager,
            split_pane_view: None,
            show_preview_panel: false,
            bookmark_selected_index: None,  // Initialize new field
            preview_focused: false,  // Initialize new field
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

    fn render(&mut self) -> Result<()> {
        // Handle special render modes
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
            NavigatorMode::SplitPane => {
                if let Some(ref mut split) = self.split_pane_view {
                    return split.render();
                }
            }
            NavigatorMode::Bookmarks => {
                return self.render_bookmarks_interface();
            }
            _ => {}
        }

        // Normal rendering with optional preview panel
        if self.show_preview_panel {
            self.render_with_preview()
        } else {
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
                search_mode: self.search_mode.as_ref(),  // Pass the search mode
                preview_focused: self.preview_focused,  // Pass the preview focus state
            };
            self.renderer.render(ctx)
        }
    }

    fn render_with_preview(&mut self) -> Result<()> {
        use crossterm::{cursor::MoveTo, execute, style::{Color, Print, ResetColor, SetForegroundColor}};
        use std::io::{self, Write};

        let mut stdout = io::stdout();
        let (terminal_width, terminal_height) = terminal::size()?;

        // Split screen: 60% for file list, 40% for preview
        let split_pos = (terminal_width as f32 * 0.6) as u16;
        let preview_width = terminal_width - split_pos - 1;

        // Render file list on the left with all the new fields
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
            search_mode: self.search_mode.as_ref(),  // Pass the search mode
            preview_focused: self.preview_focused,  // Pass the preview focus state
        };

        // Render main view (will be clipped to split_pos width)
        self.renderer.render(ctx)?;

        // Draw vertical divider
        for y in 0..terminal_height - 1 {
            execute!(
            stdout,
            MoveTo(split_pos, y),
            SetForegroundColor(Color::DarkGrey),
            Print("│"),
            ResetColor
        )?;
        }

        // Update preview based on current selection
        if let Some(entry) = self.entries.get(self.selected_index) {
            // Only reload preview if selection changed or preview is empty
            let should_reload = self.file_preview.is_none() ||
                self.file_preview.as_ref().map(|p| {
                    // Check if we need to reload (simplified check)
                    p.file_info.size == 0
                }).unwrap_or(true);

            if should_reload {
                self.file_preview = FilePreview::new(&entry.path, 50).ok();
            }
        }

        if self.file_preview.is_some() {
            self.render_preview_panel(&mut stdout, split_pos + 1, 0, preview_width, terminal_height - 1)?;
        }

        stdout.flush()?;
        Ok(())
    }

    fn render_preview_panel(
        &self,
        stdout: &mut std::io::Stdout,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) -> Result<()> {
        use crossterm::{cursor::MoveTo, execute, style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}};
        use crate::preview::{PreviewContent, FilePreview};

        if let Some(ref preview) = self.file_preview {
            // Header with file info
            execute!(
                stdout,
                MoveTo(x, y),
                SetBackgroundColor(Color::DarkBlue),
                SetForegroundColor(Color::White),
                Print(" Preview "),
                Print(" ".repeat((width - 9) as usize)),
                ResetColor
            )?;

            // File info
            execute!(
                stdout,
                MoveTo(x + 1, y + 1),
                SetForegroundColor(Color::Yellow),
                Print(format!("Size: {}", FilePreview::format_size(preview.file_info.size))),
                ResetColor
            )?;

            if let Some(perms) = preview.file_info.permissions {
                execute!(
                    stdout,
                    MoveTo(x + 1, y + 2),
                    SetForegroundColor(Color::Cyan),
                    Print(format!("Perms: {}", FilePreview::format_permissions(perms))),
                    ResetColor
                )?;
            }

            execute!(
                stdout,
                MoveTo(x + 1, y + 3),
                SetForegroundColor(Color::Green),
                Print(format!("Type: {}", preview.file_info.mime_type)),
                ResetColor
            )?;

            // Content preview
            let content_start = y + 5;
            let content_height = height.saturating_sub(6);

            match &preview.content {
                PreviewContent::Text(lines) => {
                    for (i, line) in lines.iter()
                        .skip(preview.scroll_offset)
                        .take(content_height as usize)
                        .enumerate()
                    {
                        let truncated = if line.len() > (width - 2) as usize {
                            &line[..(width - 2) as usize]
                        } else {
                            line
                        };
                        execute!(
                            stdout,
                            MoveTo(x + 1, content_start + i as u16),
                            Print(truncated)
                        )?;
                    }
                }
                PreviewContent::Binary(bytes) => {
                    execute!(
                        stdout,
                        MoveTo(x + 1, content_start),
                        SetForegroundColor(Color::DarkGrey),
                        Print("Binary file - Hex preview:"),
                        ResetColor
                    )?;

                    for (i, chunk) in bytes.chunks(16).enumerate().take((content_height - 2) as usize) {
                        let hex = chunk.iter()
                            .map(|b| format!("{:02x} ", b))
                            .collect::<String>();
                        let ascii = chunk.iter()
                            .map(|&b| if b.is_ascii_graphic() { b as char } else { '.' })
                            .collect::<String>();

                        execute!(
                            stdout,
                            MoveTo(x + 1, content_start + 2 + i as u16),
                            SetForegroundColor(Color::Blue),
                            Print(hex),
                            SetForegroundColor(Color::Green),
                            Print(" | "),
                            SetForegroundColor(Color::White),
                            Print(ascii),
                            ResetColor
                        )?;
                    }
                }
                PreviewContent::Image(info) => {
                    if let Some(ref art) = info.ascii_art {
                        for (i, line) in art.lines().enumerate().take(content_height as usize) {
                            execute!(
                                stdout,
                                MoveTo(x + 1, content_start + i as u16),
                                SetForegroundColor(Color::Magenta),
                                Print(line),
                                ResetColor
                            )?;
                        }
                    }
                }
                PreviewContent::Directory(entries) => {
                    for (i, entry) in entries.iter()
                        .skip(preview.scroll_offset)
                        .take(content_height as usize)
                        .enumerate()
                    {
                        execute!(
                            stdout,
                            MoveTo(x + 1, content_start + i as u16),
                            Print(entry)
                        )?;
                    }
                }
                PreviewContent::Error(msg) => {
                    execute!(
                        stdout,
                        MoveTo(x + 1, content_start),
                        SetForegroundColor(Color::Red),
                        Print(msg),
                        ResetColor
                    )?;
                }
                PreviewContent::Empty => {
                    execute!(
                        stdout,
                        MoveTo(x + 1, content_start),
                        SetForegroundColor(Color::DarkGrey),
                        Print("(empty file)"),
                        ResetColor
                    )?;
                }
            }
        }

        Ok(())
    }

    // In navigator.rs - complete render_bookmarks_interface method:
    fn render_bookmarks_interface(&self) -> Result<()> {
        use crossterm::{cursor::MoveTo, execute, style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor}, terminal};
        use std::io::{self, Write};

        let mut stdout = io::stdout();
        let (terminal_width, terminal_height) = terminal::size()?;

        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

        // Title
        execute!(
        stdout,
        MoveTo(0, 0),
        SetBackgroundColor(Color::DarkBlue),
        SetForegroundColor(Color::White),
        Print(" 📑 BOOKMARKS "),
        Print(" ".repeat((terminal_width - 14) as usize)),
        ResetColor
    )?;

        // Instructions
        execute!(
        stdout,
        MoveTo(2, 2),
        SetForegroundColor(Color::Yellow),
        Print("Use arrows to navigate, Enter to go, Ctrl+[letter] for quick jump"),
        ResetColor
    )?;

        // List bookmarks with selection highlight
        let bookmarks = self.bookmarks_manager.list_bookmarks();
        for (i, bookmark) in bookmarks.iter().enumerate().take((terminal_height - 5) as usize) {
            let row = 4 + i as u16;
            let is_selected = self.bookmark_selected_index == Some(i);

            let shortcut_str = bookmark.shortcut
                .map(|c| format!("[Ctrl+{}]", c))
                .unwrap_or_else(|| "        ".to_string());

            let access_str = format!("(accessed {} times)", bookmark.access_count);

            // Apply selection highlighting
            if is_selected {
                execute!(
                stdout,
                MoveTo(0, row),
                SetBackgroundColor(Color::DarkGreen),
                SetForegroundColor(Color::White),
                Print(" ".repeat(terminal_width as usize))
            )?;
                execute!(stdout, MoveTo(0, row))?;
            }

            execute!(
            stdout,
            MoveTo(2, row),
            if is_selected {
                Print("> ")
            } else {
                Print("  ")
            },
            SetForegroundColor(Color::Cyan),
            Print(shortcut_str),
            SetForegroundColor(Color::White),
            Print(format!(" {:20} ", bookmark.name)),
            SetForegroundColor(Color::Green),
            Print(format!("{:40} ", bookmark.path.display())),
            SetForegroundColor(Color::DarkGrey),
            Print(access_str),
            ResetColor
        )?;
        }

        // Available shortcuts
        let available = self.bookmarks_manager.get_available_shortcuts();
        if !available.is_empty() {
            let avail_str = available.iter()
                .take(10)
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(", ");

            execute!(
            stdout,
            MoveTo(2, terminal_height - 3),
            SetForegroundColor(Color::DarkGrey),
            Print(format!("Available shortcuts: {}", avail_str)),
            ResetColor
        )?;
        }

        // Show status message if any
        if let Some(ref msg) = self.status_message {
            execute!(
            stdout,
            MoveTo(2, terminal_height - 4),
            SetForegroundColor(Color::Yellow),
            Print(msg),
            ResetColor
        )?;
        }

        // Controls
        execute!(
        stdout,
        MoveTo(0, terminal_height - 1),
        SetBackgroundColor(Color::DarkGrey),
        SetForegroundColor(Color::White),
        Print(" ↑↓: Navigate | Enter: Go | a: Add | d: Delete | r: Rename | Ctrl+[letter]: Jump | Esc: Back "),
        Print(" ".repeat((terminal_width - 90) as usize)),
        ResetColor
    )?;

        stdout.flush()?;
        Ok(())
    }

    fn handle_input(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Result<Option<ExitAction>> {
        // Clear status message on any key press
        self.status_message = None;

        // Handle special modes first
        if self.mode == NavigatorMode::SplitPane {
            return self.handle_split_pane_input(code, modifiers);
        }

        if self.mode == NavigatorMode::Search {
            return self.handle_search_input(code, modifiers);
        }

        if self.mode == NavigatorMode::Bookmarks {
            return self.handle_bookmarks_input(code, modifiers);
        }

        match self.mode {
            NavigatorMode::Browse => {
                // Handle preview-focused controls first
                if self.show_preview_panel && self.preview_focused {
                    match code {
                        KeyCode::Up => {
                            if let Some(ref mut preview) = self.file_preview {
                                preview.scroll_up(1);
                            }
                        }
                        KeyCode::Down => {
                            if let Some(ref mut preview) = self.file_preview {
                                preview.scroll_down(1);
                            }
                        }
                        KeyCode::PageUp => {
                            if let Some(ref mut preview) = self.file_preview {
                                preview.scroll_up(10);
                            }
                        }
                        KeyCode::PageDown => {
                            if let Some(ref mut preview) = self.file_preview {
                                preview.scroll_down(10);
                            }
                        }
                        KeyCode::Tab => {
                            self.preview_focused = false;
                        }
                        KeyCode::Esc => {
                            self.preview_focused = false;
                        }
                        _ => {}
                    }
                } else {
                    // Normal browse mode controls
                    match code {
                        KeyCode::Tab if self.show_preview_panel => {
                            self.preview_focused = true;
                        }
                        KeyCode::Up => self.move_selection_up(),
                        KeyCode::Down => self.move_selection_down(),
                        KeyCode::Right | KeyCode::Enter => self.navigate_to_selected()?,
                        KeyCode::Left | KeyCode::Backspace => self.navigate_up()?,

                        // New v0.4.0 shortcuts
                        KeyCode::Char('f') if modifiers.contains(KeyModifiers::CONTROL) => {
                            self.enter_search_mode();
                        }
                        KeyCode::Char('b') if modifiers.contains(KeyModifiers::CONTROL) => {
                            self.mode = NavigatorMode::Bookmarks;
                            self.bookmark_selected_index = Some(0);
                        }
                        KeyCode::Char('g') if modifiers.contains(KeyModifiers::CONTROL) => {
                            self.show_goto_dialog()?;
                        }
                        KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => {
                            self.toggle_preview_panel();
                        }
                        KeyCode::F(2) => {
                            self.enter_split_pane_mode()?;
                        }

                        // Existing shortcuts
                        KeyCode::Char('s') if self.is_root => {
                            self.mode = NavigatorMode::Select;
                        }
                        KeyCode::Char('p') if self.is_root && !modifiers.contains(KeyModifiers::CONTROL) => {
                            self.mode = NavigatorMode::PatternSelect;
                            self.pattern_input.clear();
                        }
                        KeyCode::Char('c') if self.is_root => {
                            self.open_chmod_interface();
                        }
                        KeyCode::Char('o') if self.is_root => {
                            self.open_chown_interface();
                        }
                        KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                            return Ok(Some(ExitAction::SpawnShell(self.current_dir.clone())));
                        }
                        KeyCode::Char('S') => {
                            return Ok(Some(ExitAction::SpawnShell(self.current_dir.clone())));
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            if self.show_preview_panel {
                                self.show_preview_panel = false;
                                self.preview_focused = false;
                                self.file_preview = None;
                            } else {
                                return Ok(Some(ExitAction::Quit));
                            }
                        }
                        _ => {}
                    }
                }
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
                        let current_dir = self.current_dir.clone();
                        self.load_directory(&current_dir)?;
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn handle_search_input(
        &mut self,
        code: KeyCode,
        modifiers: KeyModifiers,
    ) -> Result<Option<ExitAction>> {
        if let Some(ref mut search) = self.search_mode {
            match code {
                KeyCode::Enter => {
                    // Execute search
                    search.search(&self.entries, &self.current_dir)?;
                    if !search.results.is_empty() {
                        self.jump_to_search_result();
                    }
                }
                KeyCode::Char('n') if modifiers.contains(KeyModifiers::CONTROL) => {
                    search.next_result();
                    self.jump_to_search_result();
                }
                KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => {
                    search.previous_result();
                    self.jump_to_search_result();
                }
                KeyCode::Char('r') if modifiers.contains(KeyModifiers::CONTROL) => {
                    search.toggle_regex();
                }
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    search.toggle_case_sensitive();
                }
                KeyCode::Char('g') if modifiers.contains(KeyModifiers::CONTROL) => {
                    search.toggle_search_contents();
                }
                KeyCode::Backspace => {
                    search.query.pop();
                }
                KeyCode::Char(c) => {
                    search.query.push(c);
                }
                KeyCode::Esc => {
                    self.mode = NavigatorMode::Browse;
                    self.search_mode = None;
                }
                _ => {}
            }
        }
        Ok(None)
    }

    fn handle_split_pane_input(
        &mut self,
        code: KeyCode,
        _modifiers: KeyModifiers,
    ) -> Result<Option<ExitAction>> {
        if let Some(ref mut split) = self.split_pane_view {
            match code {
                KeyCode::Tab => split.toggle_focus(),
                KeyCode::Up => split.get_active_pane_mut().move_up(),
                KeyCode::Down => split.get_active_pane_mut().move_down(),
                KeyCode::Enter | KeyCode::Right => {
                    split.get_active_pane_mut().navigate_to_selected()?;
                }
                KeyCode::Backspace | KeyCode::Left => {
                    split.get_active_pane_mut().navigate_up()?;
                }
                KeyCode::F(5) => split.sync_directories()?,
                KeyCode::F(6) => split.toggle_layout(),
                KeyCode::Char('+') => split.adjust_split(0.05),
                KeyCode::Char('-') => split.adjust_split(-0.05),
                KeyCode::Char(' ') => {
                    split.get_active_pane_mut().toggle_selection();
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.mode = NavigatorMode::Browse;
                    self.split_pane_view = None;
                }
                _ => {}
            }
        }
        Ok(None)
    }

    // In navigator.rs - complete handle_bookmarks_input method:
    fn handle_bookmarks_input(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<Option<ExitAction>> {
        // Initialize bookmark selection if not set
        if self.bookmark_selected_index.is_none() {
            self.bookmark_selected_index = Some(0);
        }

        let bookmarks_count = self.bookmarks_manager.list_bookmarks().len();

        match code {
            KeyCode::Up => {
                if let Some(ref mut idx) = self.bookmark_selected_index {
                    if *idx > 0 {
                        *idx -= 1;
                    }
                }
            }
            KeyCode::Down => {
                if let Some(ref mut idx) = self.bookmark_selected_index {
                    if *idx < bookmarks_count - 1 {
                        *idx += 1;
                    }
                }
            }
            KeyCode::Enter => {
                // Navigate to selected bookmark
                if let Some(idx) = self.bookmark_selected_index {
                    if let Some(bookmark) = self.bookmarks_manager.get_bookmark_by_index(idx) {
                        let path = bookmark.path.clone();
                        self.load_directory(&path)?;
                        self.mode = NavigatorMode::Browse;
                        self.bookmark_selected_index = None;
                    }
                }
            }
            KeyCode::Char('a') if !modifiers.contains(KeyModifiers::CONTROL) => {
                // Add current directory as bookmark
                let name = self.current_dir
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Bookmark")
                    .to_string();

                let available = self.bookmarks_manager.get_available_shortcuts();
                let shortcut = available.first().copied();

                if let Err(e) = self.bookmarks_manager.add_bookmark(
                    name,
                    self.current_dir.clone(),
                    shortcut,
                ) {
                    self.status_message = Some(format!("Failed to add bookmark: {}", e));
                } else {
                    self.status_message = Some("Bookmark added!".to_string());
                }
            }
            KeyCode::Char('d') if !modifiers.contains(KeyModifiers::CONTROL) => {
                // Delete selected bookmark
                if let Some(idx) = self.bookmark_selected_index {
                    if let Err(e) = self.bookmarks_manager.remove_bookmark(idx) {
                        self.status_message = Some(format!("Failed to delete bookmark: {}", e));
                    } else {
                        self.status_message = Some("Bookmark deleted!".to_string());
                        // Adjust selection if necessary
                        if idx >= bookmarks_count - 1 && idx > 0 {
                            self.bookmark_selected_index = Some(idx - 1);
                        }
                    }
                }
            }
            KeyCode::Char('r') if !modifiers.contains(KeyModifiers::CONTROL) => {
                // Rename bookmark - for now just show message
                self.status_message = Some("Rename not yet implemented".to_string());
            }
            // Use Ctrl+letter for jumping to bookmarks
            KeyCode::Char(c) if modifiers.contains(KeyModifiers::CONTROL) && c.is_alphanumeric() => {
                if let Some(bookmark) = self.bookmarks_manager.get_bookmark_by_shortcut(c) {
                    let path = bookmark.path.clone();
                    self.load_directory(&path)?;
                    self.mode = NavigatorMode::Browse;
                    self.bookmark_selected_index = None;
                }
            }
            KeyCode::Esc => {
                self.mode = NavigatorMode::Browse;
                self.bookmark_selected_index = None;
            }
            _ => {}
        }
        Ok(None)
    }

    fn enter_search_mode(&mut self) {
        self.search_mode = Some(SearchMode::new());
        self.mode = NavigatorMode::Search;
    }

    fn enter_split_pane_mode(&mut self) -> Result<()> {
        let second_path = if let Some(parent) = self.current_dir.parent() {
            parent.to_path_buf()
        } else {
            self.current_dir.clone()
        };

        self.split_pane_view = Some(SplitPaneView::new(
            self.current_dir.clone(),
            second_path,
        )?);
        self.mode = NavigatorMode::SplitPane;
        Ok(())
    }

    fn toggle_preview_panel(&mut self) {
        self.show_preview_panel = !self.show_preview_panel;
        if self.show_preview_panel {
            // Load preview for current selection
            if let Some(entry) = self.entries.get(self.selected_index) {
                self.file_preview = FilePreview::new(&entry.path, 50).ok();
            }
        } else {
            self.file_preview = None;
        }
    }

    fn show_goto_dialog(&mut self) -> Result<()> {
        // Quick bookmark jump - show numbered list
        self.mode = NavigatorMode::Bookmarks;
        Ok(())
    }

    fn jump_to_search_result(&mut self) {
        if let Some(ref search) = self.search_mode {
            if let Some(result) = search.get_current_result() {
                // Find the entry in our list
                if let Some(index) = self.entries.iter().position(|e| e.path == result.entry.path) {
                    self.selected_index = index;
                    self.adjust_scroll();
                }
            }
        }
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