use crate::AshError;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct ExecResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self, cmd: &str) -> Result<ExecResult, AshError> {
        let output = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| AshError::Msg(format!("failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(ExecResult {
            stdout,
            stderr,
            exit_code,
        })
    }

    pub fn quote(s: &str) -> String {
        let s = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`");
        format!("\"{}\"", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_success() {
        let result = Executor::new().run("echo hello").unwrap();
        assert!(result.stdout.contains("hello"));
        assert_eq!(result.exit_code, 0);
    }

    #[test]
    fn test_run_failure() {
        let result = Executor::new().run("exit 42").unwrap();
        assert_eq!(result.exit_code, 42);
    }

    #[test]
    fn test_run_stderr() {
        let result = Executor::new().run("echo error >&2").unwrap();
        assert!(result.stderr.contains("error"));
    }

    #[test]
    fn test_quote() {
        assert_eq!(Executor::quote("hello"), "\"hello\"");
        assert_eq!(Executor::quote("it's $HOME"), "\"it's \\$HOME\"");
        assert_eq!(Executor::quote("a \"b\" c"), "\"a \\\"b\\\" c\"");
    }
}
