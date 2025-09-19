use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::io;

#[allow(dead_code)]
pub fn draw_box(
    stdout: &mut io::Stdout,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    title: Option<&str>,
    color: Color,
) -> Result<()> {
    // Top border
    execute!(
        stdout,
        MoveTo(x, y),
        SetForegroundColor(color),
        Print("╭"),
        Print("─".repeat((width - 2) as usize)),
        Print("╮")
    )?;

    // Title if provided
    if let Some(title) = title {
        let title_len = title.len().min((width - 4) as usize);
        execute!(
            stdout,
            MoveTo(x + 2, y),
            Print(" "),
            Print(&title[..title_len]),
            Print(" ")
        )?;
    }

    // Side borders
    for i in 1..height - 1 {
        execute!(
            stdout,
            MoveTo(x, y + i),
            Print("│"),
            MoveTo(x + width - 1, y + i),
            Print("│")
        )?;
    }

    // Bottom border
    execute!(
        stdout,
        MoveTo(x, y + height - 1),
        Print("╰"),
        Print("─".repeat((width - 2) as usize)),
        Print("╯"),
        ResetColor
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_progress_bar(
    stdout: &mut io::Stdout,
    x: u16,
    y: u16,
    width: u16,
    progress: f32,
    color: Color,
) -> Result<()> {
    let filled = ((width as f32) * progress) as u16;

    execute!(
        stdout,
        MoveTo(x, y),
        SetForegroundColor(color),
        Print("["),
        SetBackgroundColor(color),
        Print(" ".repeat(filled as usize)),
        SetBackgroundColor(Color::Black),
        Print(" ".repeat((width - filled) as usize)),
        ResetColor,
        SetForegroundColor(color),
        Print("]"),
        ResetColor
    )?;

    Ok(())
}

#[allow(dead_code)]
pub fn draw_separator(
    stdout: &mut io::Stdout,
    y: u16,
    width: u16,
    style: SeparatorStyle,
) -> Result<()> {
    let char = match style {
        SeparatorStyle::Single => "─",
        SeparatorStyle::Double => "═",
        SeparatorStyle::Dotted => "┄",
        SeparatorStyle::Dashed => "┈",
    };

    execute!(
        stdout,
        MoveTo(0, y),
        SetForegroundColor(Color::DarkGrey),
        Print(char.repeat(width as usize)),
        ResetColor
    )?;

    Ok(())
}

#[allow(dead_code)]
pub enum SeparatorStyle {
    Single,
    Double,
    Dotted,
    Dashed,
}
