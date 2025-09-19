use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::{
    collections::HashSet,
    io::{self, Write},
    path::Path,
};

use crate::file_entry::FileEntry;
use crate::navigator::NavigatorMode;

pub struct RenderContext<'a> {
    pub current_dir: &'a Path,
    pub entries: &'a [FileEntry],
    pub selected_index: usize,
    pub selected_items: &'a HashSet<usize>,
    pub scroll_offset: usize,
    pub terminal_height: u16,
    pub mode: &'a NavigatorMode,
    pub is_root: bool,
    pub pattern_input: &'a str,
    pub status_message: &'a Option<String>,
}

pub struct Renderer {
    // Could add theme configuration here in the future
}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, ctx: RenderContext) -> Result<()> {
        let mut stdout = io::stdout();
        let (terminal_width, _) = terminal::size()?;

        // Clear screen
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        // Draw header with breadcrumb
        self.render_header(&mut stdout, ctx.current_dir, ctx.is_root, terminal_width)?;

        // Mode indicator
        self.render_mode(&mut stdout, ctx.mode, ctx.pattern_input)?;

        // Draw file list
        self.render_file_list(&mut stdout, &ctx)?;

        // Status message
        if let Some(ref msg) = ctx.status_message {
            self.render_status(&mut stdout, msg, ctx.terminal_height)?;
        }

        // Draw footer with controls
        self.render_footer(
            &mut stdout,
            ctx.mode,
            ctx.is_root,
            ctx.terminal_height,
            terminal_width,
        )?;

        stdout.flush()?;
        Ok(())
    }

    fn render_header(
        &self,
        stdout: &mut io::Stdout,
        current_dir: &Path,
        is_root: bool,
        terminal_width: u16,
    ) -> Result<()> {
        let header_text = if is_root {
            format!(" 📂 {} [ROOT MODE]", current_dir.display())
        } else {
            format!(" 📂 {}", current_dir.display())
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

        Ok(())
    }

    fn render_mode(
        &self,
        stdout: &mut io::Stdout,
        mode: &NavigatorMode,
        pattern_input: &str,
    ) -> Result<()> {
        let mode_text = match mode {
            NavigatorMode::Browse => "BROWSE".to_string(),
            NavigatorMode::Select => "SELECT (Space: toggle, Enter: confirm)".to_string(),
            NavigatorMode::PatternSelect => format!("PATTERN: {}_", pattern_input),
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

        Ok(())
    }

    fn render_file_list(&self, stdout: &mut io::Stdout, ctx: &RenderContext) -> Result<()> {
        let (terminal_width, _) = terminal::size()?;
        let list_start = 3;
        let visible_area = (ctx.terminal_height as usize).saturating_sub(5);
        let end_index = (ctx.scroll_offset + visible_area).min(ctx.entries.len());

        for (i, entry) in ctx.entries[ctx.scroll_offset..end_index].iter().enumerate() {
            let row = (list_start + i) as u16;
            execute!(stdout, MoveTo(0, row))?;

            let display_index = ctx.scroll_offset + i;
            let is_selected = ctx.selected_items.contains(&display_index);
            let is_highlighted = display_index == ctx.selected_index;

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
            if *ctx.mode == NavigatorMode::Select {
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
            if *ctx.mode == NavigatorMode::Select && ctx.is_root {
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
                    + if *ctx.mode == NavigatorMode::Select {
                        4
                    } else {
                        0
                    }
                    + if *ctx.mode == NavigatorMode::Select && ctx.is_root {
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

        Ok(())
    }

    fn render_status(
        &self,
        stdout: &mut io::Stdout,
        msg: &str,
        terminal_height: u16,
    ) -> Result<()> {
        let status_row = terminal_height - 2;
        execute!(
            stdout,
            MoveTo(0, status_row),
            SetForegroundColor(Color::Yellow),
            Print(format!(" {} ", msg)),
            ResetColor
        )?;
        Ok(())
    }

    fn render_footer(
        &self,
        stdout: &mut io::Stdout,
        mode: &NavigatorMode,
        is_root: bool,
        terminal_height: u16,
        terminal_width: u16,
    ) -> Result<()> {
        let footer_row = terminal_height - 1;
        let controls = if is_root {
            match mode {
                NavigatorMode::Browse => {
                    " ↑↓:Navigate  →/Enter:Open  ←:Up  s:Select  p:Pattern  c:Chmod  S/Ctrl+D:Shell  q:Quit"
                }
                NavigatorMode::Select => {
                    " ↑↓:Navigate  Space:Toggle  Enter:Confirm  c:Chmod  Esc:Cancel"
                }
                NavigatorMode::PatternSelect => {
                    " Type pattern: 'ali*' for prefix, '.*log' for suffix, '^ali' for exact prefix | Enter:Apply | Esc:Cancel"
                }
                _ => "",
            }
        } else {
            " ↑↓:Navigate  →/Enter:Open  ←/Backspace:Up  S/Ctrl+D:Shell  Esc/q:Quit"
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

        Ok(())
    }
}
