use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{env, io, process::Command};

// Core modules
mod managers;
mod models;
mod navigator;
mod ui;
mod utils;

// v0.4.0 Enhanced Navigation modules
mod bookmarks;
mod preview;
mod search;
mod split_pane;

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

fn print_version() {
    println!("fsnav v0.4.0 - Enhanced Navigation Edition");
    println!("A fast terminal file system navigator written in Rust");
    println!("\nNew features in v0.4.0:");
    println!("  • Search with Ctrl+F (regex support)");
    println!("  • File preview panel with Ctrl+P");
    println!("  • Bookmarks system with Ctrl+B");
    println!("  • Split-pane view with F2");
    println!("\nFor more information, visit: https://github.com/AlexArtaud-Dev/fsnav");
}

fn print_help() {
    println!("Usage: fsnav [OPTIONS] [PATH]");
    println!("\nOptions:");
    println!("  -h, --help     Show this help message");
    println!("  -v, --version  Show version information");
    println!("  PATH           Start in the specified directory");
    println!("\nKeyboard Shortcuts:");
    println!("\nNavigation:");
    println!("  ↑/↓           Navigate up/down");
    println!("  →/Enter       Enter directory");
    println!("  ←/Backspace   Go to parent directory");
    println!("  S/Ctrl+D      Spawn shell in current directory");
    println!("  Esc/q         Quit");
    println!("\nSearch & Preview:");
    println!("  Ctrl+F        Search files (supports regex)");
    println!("  Ctrl+N/P      Next/Previous search result");
    println!("  Ctrl+P        Toggle preview panel");
    println!("  F2            Split-pane view");
    println!("\nBookmarks:");
    println!("  Ctrl+B        Open bookmarks");
    println!("  Ctrl+G        Quick jump to bookmark");
    println!("\nRoot Mode (when running as root):");
    println!("  s             Selection mode");
    println!("  p             Pattern selection");
    println!("  c             Chmod interface");
    println!("  o             Chown interface");
}

#[cfg(windows)]
fn main() {
    eprintln!("❌ fsnav does not support Windows directly. Please use WSL.");
    std::process::exit(1);
}

#[cfg(not(windows))]
fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    if args.len() > 1 {
        match args[1].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(());
            }
            "-v" | "--version" => {
                print_version();
                return Ok(());
            }
            path => {
                // Try to start in the specified directory
                let target_path = std::path::Path::new(path);
                if target_path.exists() && target_path.is_dir() {
                    env::set_current_dir(target_path)?;
                } else {
                    eprintln!("Error: '{}' is not a valid directory", path);
                    std::process::exit(1);
                }
            }
        }
    }

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