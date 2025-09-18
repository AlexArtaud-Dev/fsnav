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
    os::unix::fs::PermissionsExt,
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct ChmodInterface {
    // Current chmod value as 3 digits (e.g., [7, 5, 5] for 755)
    digits: [u8; 3],
    // Current position (0=owner, 1=group, 2=others)
    position: usize,
    // Selected files/directories
    selected_paths: Vec<PathBuf>,
    // Preview mode
    preview_mode: bool,
    // Template mode
    show_templates: bool,
    template_index: usize,
}

impl ChmodInterface {
    pub fn new(selected_paths: Vec<PathBuf>) -> Self {
        // Try to get current permissions from first file
        let initial_digits = if let Some(first_path) = selected_paths.first() {
            if let Ok(metadata) = first_path.metadata() {
                let mode = metadata.permissions().mode();
                [
                    ((mode >> 6) & 0b111) as u8,
                    ((mode >> 3) & 0b111) as u8,
                    (mode & 0b111) as u8,
                ]
            } else {
                [6, 4, 4] // Default
            }
        } else {
            [6, 4, 4]
        };

        Self {
            digits: initial_digits,
            position: 0,
            selected_paths,
            preview_mode: true,
            show_templates: false,
            template_index: 0,
        }
    }

    pub fn render(&self) -> Result<()> {
        let mut stdout = io::stdout();
        let (_terminal_width, _) = terminal::size()?;

        // Clear and setup
        execute!(stdout, terminal::Clear(terminal::ClearType::All))?;

        // Title
        execute!(
            stdout,
            MoveTo(0, 0),
            SetForegroundColor(Color::Cyan),
            Print("╔═══════════════════════════════════════════════════════════════════════╗"),
            MoveTo(0, 1),
            Print("║           INTERACTIVE CHMOD - Permission Manager                     ║"),
            MoveTo(0, 2),
            Print("╚═══════════════════════════════════════════════════════════════════════╝"),
            ResetColor
        )?;

        // Selected files
        execute!(
            stdout,
            MoveTo(0, 4),
            SetForegroundColor(Color::Yellow),
            Print(format!(
                "📁 Selected: {} item(s)",
                self.selected_paths.len()
            )),
            ResetColor
        )?;

        // Show first few selected paths
        for (i, path) in self.selected_paths.iter().take(2).enumerate() {
            let display_path = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path.to_str().unwrap_or("?"));
            execute!(
                stdout,
                MoveTo(3, 5 + i as u16),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("• {}", display_path)),
                ResetColor
            )?;
        }

        if self.selected_paths.len() > 2 {
            execute!(
                stdout,
                MoveTo(3, 7),
                SetForegroundColor(Color::DarkGrey),
                Print(format!("  ... and {} more", self.selected_paths.len() - 2)),
                ResetColor
            )?;
        }

        if self.show_templates {
            self.render_templates(&mut stdout)?;
        } else {
            // Chmod selector interface
            self.render_chmod_selector(&mut stdout, 9)?;

            // Permission preview
            self.render_permission_preview(&mut stdout, 16)?;

            // Explanation
            self.render_explanation(&mut stdout, 20)?;
        }

        // Controls
        self.render_controls(&mut stdout, 26)?;

        stdout.flush()?;
        Ok(())
    }

    fn render_templates(&self, stdout: &mut io::Stdout) -> Result<()> {
        execute!(
            stdout,
            MoveTo(5, 9),
            SetForegroundColor(Color::Cyan),
            Print("📋 PERMISSION TEMPLATES"),
            ResetColor
        )?;

        let templates = [
            ("755", "Standard (rwxr-xr-x)", "Executables and directories"),
            ("644", "Read Only (rw-r--r--)", "Regular files"),
            ("600", "Private (rw-------)", "Sensitive files, owner only"),
            (
                "700",
                "Private Exec (rwx------)",
                "Private scripts/directories",
            ),
            ("775", "Group Share (rwxrwxr-x)", "Shared directories"),
            ("664", "Group Write (rw-rw-r--)", "Collaborative files"),
            ("666", "All Write (rw-rw-rw-)", "Temporary/log files"),
            (
                "777",
                "Full Access (rwxrwxrwx)",
                "⚠️ DANGEROUS - Everyone has full access",
            ),
            ("400", "Read Only Owner (r--------)", "Protected configs"),
            ("500", "Exec Only Owner (r-x------)", "Protected scripts"),
        ];

        for (i, (value, name, desc)) in templates.iter().enumerate() {
            let is_selected = i == self.template_index;
            let y = 11 + i as u16;

            execute!(stdout, MoveTo(5, y))?;

            if is_selected {
                execute!(
                    stdout,
                    SetBackgroundColor(Color::DarkGreen),
                    SetForegroundColor(Color::White),
                    Print(" > ")
                )?;
            } else {
                execute!(stdout, Print("   "))?;
            }

            execute!(
                stdout,
                SetForegroundColor(if is_selected {
                    Color::White
                } else {
                    Color::Grey
                }),
                Print(format!("{} ", value)),
                SetForegroundColor(if is_selected {
                    Color::Yellow
                } else {
                    Color::DarkGrey
                }),
                Print(format!("{:<18} ", name)),
                SetForegroundColor(if is_selected {
                    Color::Cyan
                } else {
                    Color::DarkGrey
                }),
                Print(desc),
                ResetColor
            )?;
        }

        Ok(())
    }

    fn render_chmod_selector(&self, stdout: &mut io::Stdout, y: u16) -> Result<()> {
        execute!(
            stdout,
            MoveTo(8, y),
            SetForegroundColor(Color::Cyan),
            Print("╭─────────────────────────────────────────────╮"),
            MoveTo(8, y + 1),
            Print("│         OWNER      GROUP      OTHERS        │"),
            MoveTo(8, y + 2),
            Print("├─────────────────────────────────────────────┤"),
            ResetColor
        )?;

        // Render the three digit selectors with visual indicators
        for (i, digit) in self.digits.iter().enumerate() {
            let x = 17 + (i as u16 * 12); // Adjusted for better centering
            let is_selected = i == self.position;

            // Draw the selector box
            execute!(stdout, MoveTo(x - 2, y + 3))?;

            if is_selected {
                // Animated selection box
                execute!(
                    stdout,
                    SetForegroundColor(Color::Green),
                    Print("┌───┐"),
                    MoveTo(x - 2, y + 4),
                    Print("│"),
                    MoveTo(x + 2, y + 4),
                    Print("│"),
                    MoveTo(x - 2, y + 5),
                    Print("└───┘"),
                    ResetColor
                )?;

                // Up/down arrows
                execute!(
                    stdout,
                    MoveTo(x, y + 2),
                    SetForegroundColor(Color::Green),
                    Print("▲"),
                    MoveTo(x, y + 6),
                    Print("▼"),
                    ResetColor
                )?;
            }

            // Draw the digit
            execute!(
                stdout,
                MoveTo(x, y + 4),
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
                Print(format!(" {} ", digit)),
                ResetColor
            )?;
        }

        execute!(
            stdout,
            MoveTo(8, y + 7),
            SetForegroundColor(Color::Cyan),
            Print("╰─────────────────────────────────────────────╯"),
            ResetColor
        )?;

        Ok(())
    }

    fn render_permission_preview(&self, stdout: &mut io::Stdout, y: u16) -> Result<()> {
        let mode_value = format!("{}{}{}", self.digits[0], self.digits[1], self.digits[2]);

        execute!(
            stdout,
            MoveTo(5, y),
            SetForegroundColor(Color::Yellow),
            Print("📊 Permission Preview:"),
            ResetColor
        )?;

        // Visual representation with colors
        let visual = self.get_visual_permissions();
        execute!(stdout, MoveTo(8, y + 1))?;

        // Draw permission blocks
        for (group_idx, group) in visual.chars().collect::<Vec<_>>().chunks(3).enumerate() {
            let (label, color) = match group_idx {
                0 => ("Owner:", Color::Red),
                1 => ("Group:", Color::Yellow),
                2 => ("Other:", Color::Green),
                _ => ("", Color::White),
            };

            execute!(
                stdout,
                SetForegroundColor(color),
                Print(format!("{:<7}", label)),
                ResetColor
            )?;

            for &ch in group {
                let (symbol, active) = match ch {
                    'r' => ("R", true),
                    'w' => ("W", true),
                    'x' => ("X", true),
                    _ => ("─", false),
                };

                if active {
                    execute!(
                        stdout,
                        SetBackgroundColor(color),
                        SetForegroundColor(Color::Black),
                        Print(format!(" {} ", symbol)),
                        ResetColor,
                        Print(" ")
                    )?;
                } else {
                    execute!(stdout, SetForegroundColor(Color::DarkGrey), Print(" ─  "))?;
                }
            }

            if group_idx < 2 {
                execute!(stdout, Print("  "))?;
            }
        }

        // Octal value
        execute!(
            stdout,
            MoveTo(8, y + 2),
            SetForegroundColor(Color::Cyan),
            Print("Octal: "),
            SetForegroundColor(Color::White),
            Print(format!("{} ", mode_value)),
            SetForegroundColor(Color::DarkGrey),
            Print(format!(
                "(Binary: {:03b} {:03b} {:03b})",
                self.digits[0], self.digits[1], self.digits[2]
            )),
            ResetColor
        )?;

        Ok(())
    }

    fn render_explanation(&self, stdout: &mut io::Stdout, y: u16) -> Result<()> {
        execute!(
            stdout,
            MoveTo(5, y),
            SetForegroundColor(Color::Cyan),
            Print("💡 What this means:"),
            ResetColor
        )?;

        let explanations = self.get_explanations();
        for (i, explanation) in explanations.iter().enumerate() {
            let (icon, color) = match i {
                0 => ("👤", Color::Red),
                1 => ("👥", Color::Yellow),
                2 => ("🌍", Color::Green),
                3 => ("ℹ️", Color::Cyan),
                _ => ("•", Color::White),
            };

            execute!(
                stdout,
                MoveTo(8, y + 1 + i as u16),
                SetForegroundColor(color),
                Print(format!("{} ", icon)),
                SetForegroundColor(Color::White),
                Print(explanation),
                ResetColor
            )?;
        }

        Ok(())
    }

    fn render_controls(&self, stdout: &mut io::Stdout, y: u16) -> Result<()> {
        let controls = if self.show_templates {
            " ↑↓: Select Template | Enter: Apply | t: Manual Mode | Esc: Cancel "
        } else {
            " ←→: Navigate | ↑↓: Change | t: Templates | Enter: Apply | Esc: Cancel "
        };

        execute!(
            stdout,
            MoveTo(0, y),
            SetBackgroundColor(Color::DarkGrey),
            SetForegroundColor(Color::White),
            Print(controls),
            ResetColor
        )?;

        if self.preview_mode {
            execute!(
                stdout,
                MoveTo(0, y + 1),
                SetBackgroundColor(Color::DarkYellow),
                SetForegroundColor(Color::Black),
                Print(" ⚠️  PREVIEW MODE - Changes will be applied to all selected items "),
                ResetColor
            )?;
        }

        Ok(())
    }

    fn get_visual_permissions(&self) -> String {
        let mut result = String::new();

        for digit in &self.digits {
            result.push(if digit & 4 != 0 { 'r' } else { '-' });
            result.push(if digit & 2 != 0 { 'w' } else { '-' });
            result.push(if digit & 1 != 0 { 'x' } else { '-' });
        }

        result
    }

    fn get_explanations(&self) -> Vec<String> {
        let mut explanations = Vec::new();

        // Owner permissions
        let owner_perms = self.digit_to_permissions(self.digits[0]);
        explanations.push(format!("Owner can: {}", owner_perms));

        // Group permissions
        let group_perms = self.digit_to_permissions(self.digits[1]);
        explanations.push(format!("Group members can: {}", group_perms));

        // Others permissions
        let others_perms = self.digit_to_permissions(self.digits[2]);
        explanations.push(format!("Everyone else can: {}", others_perms));

        // Security assessment
        let pattern = format!("{}{}{}", self.digits[0], self.digits[1], self.digits[2]);
        let security = match pattern.as_str() {
            "777" => "⚠️ VERY INSECURE - Anyone can do anything!",
            "666" => "⚠️ Risky - Anyone can modify these files",
            "755" => "✓ Standard - Safe for programs and directories",
            "644" => "✓ Standard - Safe for regular files",
            "600" => "✓ Secure - Only you have access",
            "700" => "✓ Secure - Private directory/executable",
            "000" => "⚠️ Locked - Nobody can access (unusual)",
            _ => {
                let world_write = self.digits[2] & 2 != 0;
                if world_write {
                    "⚠️ World-writable - Consider restricting"
                } else {
                    "Custom permissions set"
                }
            }
        };
        explanations.push(security.to_string());

        explanations
    }

    fn digit_to_permissions(&self, digit: u8) -> String {
        let mut perms = Vec::new();

        if digit & 4 != 0 {
            perms.push("read");
        }
        if digit & 2 != 0 {
            perms.push("write");
        }
        if digit & 1 != 0 {
            perms.push("execute/enter");
        }

        if perms.is_empty() {
            "nothing (no access)".to_string()
        } else {
            perms.join(", ")
        }
    }

    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        if self.show_templates {
            match key {
                KeyCode::Up => {
                    if self.template_index > 0 {
                        self.template_index -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.template_index < 9 {
                        self.template_index += 1;
                    }
                }
                KeyCode::Enter => {
                    // Apply template
                    let templates = [
                        [7, 5, 5], // 755
                        [6, 4, 4], // 644
                        [6, 0, 0], // 600
                        [7, 0, 0], // 700
                        [7, 7, 5], // 775
                        [6, 6, 4], // 664
                        [6, 6, 6], // 666
                        [7, 7, 7], // 777
                        [4, 0, 0], // 400
                        [5, 0, 0], // 500
                    ];
                    self.digits = templates[self.template_index];
                    self.apply_permissions();
                    return false; // Exit interface
                }
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    self.show_templates = false;
                }
                KeyCode::Esc => {
                    return false; // Exit without applying
                }
                _ => {}
            }
        } else {
            match key {
                KeyCode::Left => {
                    if self.position > 0 {
                        self.position -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.position < 2 {
                        self.position += 1;
                    }
                }
                KeyCode::Up => {
                    if self.digits[self.position] < 7 {
                        self.digits[self.position] += 1;
                    }
                }
                KeyCode::Down => {
                    if self.digits[self.position] > 0 {
                        self.digits[self.position] -= 1;
                    }
                }
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    self.show_templates = true;
                    self.template_index = 0;
                }
                KeyCode::Enter => {
                    self.apply_permissions();
                    return false; // Exit interface
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    self.preview_mode = !self.preview_mode;
                }
                KeyCode::Esc => {
                    return false; // Exit without applying
                }
                _ => {}
            }
        }
        true // Continue
    }

    fn apply_permissions(&self) {
        let mode =
            (self.digits[0] as u32) * 64 + (self.digits[1] as u32) * 8 + (self.digits[2] as u32);

        for path in &self.selected_paths {
            if path.exists() {
                #[cfg(unix)]
                {
                    if let Ok(metadata) = path.metadata() {
                        let mut permissions = metadata.permissions();
                        permissions.set_mode(0o100000 | mode); // Preserve file type bits
                        let _ = std::fs::set_permissions(path, permissions);
                    }
                }
            }
        }
    }
}
