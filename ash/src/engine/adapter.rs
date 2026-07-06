use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use super::driver::LocalCliDriver;
use super::types::{ExecuteRequest, ExecuteResponse};

pub trait Adapter: Send + Sync {
    fn name(&self) -> &str;
    fn execute(&self, req: &ExecuteRequest) -> ExecuteResponse;
}

pub struct LocalCliAdapter {
    name: String,
    driver: Arc<dyn LocalCliDriver>,
}

impl LocalCliAdapter {
    pub fn new(name: &str, driver: Arc<dyn LocalCliDriver>) -> Self {
        LocalCliAdapter {
            name: name.to_string(),
            driver,
        }
    }
}

impl Adapter for LocalCliAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, req: &ExecuteRequest) -> ExecuteResponse {
        let spec = self.driver.build_command(req);

        {
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/tmp/ash-commands.log")
                .unwrap();
            writeln!(f, "[ash:{}] {} {}", self.name, spec.cmd, spec.args.join(" ")).unwrap();
        }

        let mut child = match Command::new(&spec.cmd)
            .args(&spec.args)
            .stdin(if spec.stdin_prompt {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                return ExecuteResponse {
                    stdout: String::new(),
                    stderr: format!("failed to spawn {}: {}", spec.cmd, e),
                    exit_code: -1,
                };
            }
        };

        if spec.stdin_prompt {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(req.prompt.as_bytes());
            }
        }

        let stdout_buf = Arc::new(Mutex::new(String::new()));
        let stderr_buf = Arc::new(Mutex::new(String::new()));

        let stdout_handle = if let Some(out) = child.stdout.take() {
            let buf = stdout_buf.clone();
            Some(thread::spawn(move || {
                let reader = BufReader::new(out);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            let mut b = buf.lock().unwrap();
                            b.push_str(&l);
                            b.push('\n');
                            let _ = std::io::stdout().write_all(l.as_bytes());
                            let _ = std::io::stdout().write_all(b"\n");
                            let _ = std::io::stdout().flush();
                        }
                        Err(_) => break,
                    }
                }
            }))
        } else {
            None
        };

        let stderr_handle = if let Some(err) = child.stderr.take() {
            let buf = stderr_buf.clone();
            Some(thread::spawn(move || {
                let reader = BufReader::new(err);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            let mut b = buf.lock().unwrap();
                            b.push_str(&l);
                            b.push('\n');
                            let _ = std::io::stderr().write_all(l.as_bytes());
                            let _ = std::io::stderr().write_all(b"\n");
                            let _ = std::io::stderr().flush();
                        }
                        Err(_) => break,
                    }
                }
            }))
        } else {
            None
        };

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

        let stdout = stdout_buf.lock().unwrap().clone();
        let stderr = stderr_buf.lock().unwrap().clone();

        ExecuteResponse {
            stdout,
            stderr,
            exit_code,
        }
    }
}
