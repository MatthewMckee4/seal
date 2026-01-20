use anyhow::{Context, Result};
use std::{path::Path, process::Command};

pub struct CommandWrapper {
    /// The command to execute.
    ///
    /// Like `["git", "add", "-A"]`
    command_with_args: Vec<String>,
}

impl CommandWrapper {
    /// Create a new command.
    pub fn new<T>(command_with_args: Vec<T>) -> Self
    where
        T: ToString,
    {
        Self {
            command_with_args: command_with_args
                .into_iter()
                .map(|arg| arg.to_string())
                .collect(),
        }
    }

    pub fn as_string(&self) -> String {
        self.command_with_args.join(" ")
    }

    /// Execute the command.
    pub fn execute(
        &self,
        stdout: &mut dyn std::fmt::Write,
        current_directory: &Path,
    ) -> Result<()> {
        let command_str = self.as_string();
        writeln!(stdout, "Executing command: `{command_str}`")?;

        Command::new(&self.command_with_args[0])
            .args(&self.command_with_args[1..])
            .current_dir(current_directory)
            .output()
            .context("Failed to execute `{command_str}`")?;

        Ok(())
    }

    pub fn git_add_all() -> Self {
        Self::new(vec!["git", "add", "-A"])
    }

    pub fn git_commit(message: &str) -> Self {
        Self::new(vec!["git", "commit", "-m", message])
    }

    pub fn create_branch(name: &str) -> Self {
        Self::new(vec!["git", "checkout", "-b", name])
    }

    pub fn git_push_branch(branch_name: &str) -> Self {
        Self::new(vec!["git", "push", "origin", branch_name])
    }

    /// Create a custom command from a shell command string.
    ///
    /// The command string is split on whitespace. For complex commands with
    /// quoted arguments, consider using `new` directly with a properly
    /// constructed argument vector.
    pub fn custom(command: &str) -> Self {
        let parts: Vec<&str> = command.split_whitespace().collect();
        Self::new(parts)
    }
}
