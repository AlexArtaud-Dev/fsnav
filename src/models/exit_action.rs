use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum ExitAction {
    Quit,
    SpawnShell(PathBuf),
}
