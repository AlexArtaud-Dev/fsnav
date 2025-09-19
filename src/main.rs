use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{env, io, process::Command};

mod managers;
mod models;
mod navigator;
mod ui;
mod utils;

use models::ExitAction;
use navigator::Navigator;

fn run_app() -> Result<ExitAction> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let mut nav = Navigator::new()?;
    let exit_action = nav.run()?;

    execute!(stdout, LeaveAlternateScreen, Show)?;
    terminal::disable_raw_mode()?;

    Ok(exit_action)
}

fn spawn_shell_in_directory(dir: &std::path::Path) -> Result<()> {
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    println!("📂 Spawning new shell in: {}", dir.display());
    println!("Type 'exit' to return to the original directory\n");

    let status = Command::new(&shell)
        .current_dir(dir)
        .status()
        .context("Failed to spawn shell")?;

    if !status.success() {
        eprintln!("Shell exited with status: {:?}", status);
    }

    Ok(())
}

#[cfg(windows)]
fn main() {
    eprintln!("❌ fsnav does not support Windows directly. Please use WSL.");
    std::process::exit(1);
}

#[cfg(not(windows))]
fn main() -> Result<()> {
    let result = run_app();

    let mut stdout = io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen, Show);
    let _ = terminal::disable_raw_mode();

    match result {
        Ok(ExitAction::SpawnShell(dir)) => {
            spawn_shell_in_directory(&dir)?;
        }
        Ok(ExitAction::Quit) => {}
        Err(e) => return Err(e),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        assert!(true);
    }
}
