use anyhow::{Context, Result, bail};
use std::{path::Path, process::Command};

/// Result of executing a command.
#[derive(Debug)]
pub struct CommandResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stderr: String,
}

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

    /// Execute the command and return an error if it fails.
    pub fn execute(
        &self,
        stdout: &mut dyn std::fmt::Write,
        current_directory: &Path,
    ) -> Result<()> {
        let result = self.execute_with_result(stdout, current_directory)?;
        if !result.success {
            let exit_info = result
                .exit_code
                .map(|code| format!(" (exit code {code})"))
                .unwrap_or_default();
            let stderr_info = if result.stderr.is_empty() {
                String::new()
            } else {
                format!("\n{}", result.stderr.trim())
            };
            bail!(
                "Command `{}` failed{exit_info}{stderr_info}",
                self.as_string()
            );
        }
        Ok(())
    }

    /// Execute the command and return the result without failing on non-zero exit.
    pub fn execute_with_result(
        &self,
        stdout: &mut dyn std::fmt::Write,
        current_directory: &Path,
    ) -> Result<CommandResult> {
        let command_str = self.as_string();
        writeln!(stdout, "Executing command: `{command_str}`")?;

        let output = Command::new(&self.command_with_args[0])
            .args(&self.command_with_args[1..])
            .current_dir(current_directory)
            .output()
            .with_context(|| format!("Failed to execute `{command_str}`"))?;

        Ok(CommandResult {
            success: output.status.success(),
            exit_code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
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
