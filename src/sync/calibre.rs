use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug)]
pub struct CalibreHook {
    binary_path: PathBuf,
}

impl CalibreHook {
    pub fn new(binary_path: impl Into<PathBuf>) -> Self {
        Self {
            binary_path: binary_path.into(),
        }
    }

    pub fn add_book_command(&self, path: &str) -> Command {
        let mut command = Command::new(&self.binary_path);
        command.arg("add").arg(path);
        command
    }
}
