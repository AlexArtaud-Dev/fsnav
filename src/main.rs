use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    env,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct FileEntry {
    name: String,
    path: PathBuf,
    is_dir: bool,
    is_accessible: bool,
}

impl FileEntry {
    fn display_name(&self) -> String {
        if self.is_dir {
            format!("📁 {}/", self.name)
        } else {
            format!("📄 {}", self.name)
        }
    }
}

struct Navigator {
    current_dir: PathBuf,
    entries: Vec<FileEntry>,
    selected_index: usize,
    scroll_offset: usize,
    terminal_height: u16,
}

impl Navigator {
    fn new() -> Result<Self> {
        let current_dir = env::current_dir().context("Failed to get current directory")?;
        let mut nav = Self {
            current_dir: current_dir.clone(),
            entries: Vec::new(),
            selected_index: 0,
            scroll_offset: 0,
            terminal_height: terminal::size()?.1,
        };
        nav.load_directory(&current_dir)?;
        Ok(nav)
    }

    fn load_directory(&mut self, path: &Path) -> Result<()> {
        self.entries.clear();
        self.selected_index = 0;
        self.scroll_offset = 0;

        // Add parent directory entry if not at root
        if let Some(parent) = path.parent() {
            if parent != path {
                self.entries.push(FileEntry {
                    name: "..".to_string(),
                    path: parent.to_path_buf(),
                    is_dir: true,
                    is_accessible: true,
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
                    
                    let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                    let is_accessible = metadata.is_ok();
                    
                    let name = entry
                        .file_name()
                        .to_string_lossy()
                        .to_string();

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

    fn adjust_scroll(&mut self) {
        let visible_area = (self.terminal_height as usize).saturating_sub(4); // Header + footer space
        
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_area {
            self.scroll_offset = self.selected_index.saturating_sub(visible_area - 1);
        }
    }

    fn render(&self) -> Result<()> {
        let mut stdout = io::stdout();
        let (terminal_width, terminal_height) = terminal::size()?;

        // Clear screen
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        // Draw header with breadcrumb
        execute!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            Print(" ".repeat(terminal_width as usize)),
            MoveTo(0, 0),
            Print(format!(" 📂 {}", self.current_dir.display())),
            ResetColor
        )?;

        // Draw file list
        let visible_area = (terminal_height as usize).saturating_sub(4);
        let end_index = (self.scroll_offset + visible_area).min(self.entries.len());

        for (i, entry) in self.entries[self.scroll_offset..end_index].iter().enumerate() {
            let row = i as u16 + 2;
            execute!(stdout, MoveTo(0, row))?;

            let display_index = self.scroll_offset + i;
            if display_index == self.selected_index {
                execute!(
                    stdout,
                    SetBackgroundColor(Color::DarkGrey),
                    SetForegroundColor(Color::White),
                    Print(format!(" > {}", entry.display_name())),
                    Print(" ".repeat(
                        terminal_width as usize - entry.display_name().len() - 4
                    )),
                    ResetColor
                )?;
            } else {
                let color = if !entry.is_accessible {
                    Color::DarkRed
                } else if entry.is_dir {
                    Color::Cyan
                } else {
                    Color::White
                };
                
                execute!(
                    stdout,
                    SetForegroundColor(color),
                    Print(format!("   {}", entry.display_name())),
                    ResetColor
                )?;
            }
        }

        // Draw footer with controls
        let footer_row = terminal_height - 1;
        execute!(
            stdout,
            MoveTo(0, footer_row),
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
            Print(" ↑↓:Navigate  →/Enter:Open  ←/Backspace:Up  Esc/q:Quit"),
            Print(" ".repeat(
                terminal_width as usize - 56
            )),
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
            }) = event::read()? {
                match code {
                    KeyCode::Up => nav.move_selection_up(),
                    KeyCode::Down => nav.move_selection_down(),
                    KeyCode::Right | KeyCode::Enter => nav.navigate_to_selected()?,
                    KeyCode::Left | KeyCode::Backspace => nav.navigate_up()?,
                    KeyCode::Esc | KeyCode::Char('q') => break,
                    _ => {}
                }
            }
        }
    }

    Ok(())
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
    use std::fs;
    use std::path::Path;

    #[test]
    fn test_file_entry_display() {
        let dir_entry = FileEntry {
            name: "test_dir".to_string(),
            path: PathBuf::from("/test/test_dir"),
            is_dir: true,
            is_accessible: true,
        };
        assert_eq!(dir_entry.display_name(), "📁 test_dir/");

        let file_entry = FileEntry {
            name: "test.txt".to_string(),
            path: PathBuf::from("/test/test.txt"),
            is_dir: false,
            is_accessible: true,
        };
        assert_eq!(file_entry.display_name(), "📄 test.txt");
    }
}
