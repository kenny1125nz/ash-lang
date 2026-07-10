use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use crate::util::lock_guard;
use crate::AshError;

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

    pub fn run_forwarded(&self, cmd: &str) -> Result<ExecResult, AshError> {
        let mut child = Command::new("bash")
            .arg("-c")
            .arg(cmd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AshError::Msg(format!("failed to execute command: {}", e)))?;

        let stdout_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let stderr_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

        let stdout_handle = child.stdout.take().map(|out| {
            let buf = stdout_buf.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(out);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            let mut b = lock_guard(&buf);
                            b.push_str(&l);
                            b.push('\n');
                            let _ = std::io::stdout().write_all(l.as_bytes());
                            let _ = std::io::stdout().write_all(b"\n");
                            let _ = std::io::stdout().flush();
                        }
                        Err(_) => break,
                    }
                }
            })
        });

        let stderr_handle = child.stderr.take().map(|err| {
            let buf = stderr_buf.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(err);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            let mut b = lock_guard(&buf);
                            b.push_str(&l);
                            b.push('\n');
                            let _ = std::io::stderr().write_all(l.as_bytes());
                            let _ = std::io::stderr().write_all(b"\n");
                            let _ = std::io::stderr().flush();
                        }
                        Err(_) => break,
                    }
                }
            })
        });

        let exit_code = match child.wait() {
            Ok(status) => status.code().unwrap_or(-1),
            Err(_) => -1,
        };

        if let Some(h) = stdout_handle {
            let _ = h.join();
        }
        if let Some(h) = stderr_handle {
            let _ = h.join();
        }

        let stdout = lock_guard(&stdout_buf).clone();
        let stderr = lock_guard(&stderr_buf).clone();

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
