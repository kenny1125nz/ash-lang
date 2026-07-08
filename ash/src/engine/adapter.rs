use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

use crate::util::{get_pool, lock_guard};

use log::{debug, info, trace};

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

        info!("agent — connecting to {} at {}", self.name, spec.cmd);
        debug!("agent — request: {} {}", spec.cmd, spec.args.join(" "));

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

        let (stdout_tx, stdout_rx) = mpsc::channel();
        let (stderr_tx, stderr_rx) = mpsc::channel();

        if let Some(out) = child.stdout.take() {
            let buf = stdout_buf.clone();
            get_pool().execute(move || {
                let reader = BufReader::new(out);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            trace!("agent stdout: {}", l);
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
                let _ = stdout_tx.send(());
            });
        } else {
            let _ = stdout_tx.send(());
        }

        if let Some(err) = child.stderr.take() {
            let buf = stderr_buf.clone();
            get_pool().execute(move || {
                let reader = BufReader::new(err);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            trace!("agent stderr: {}", l);
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
                let _ = stderr_tx.send(());
            });
        } else {
            let _ = stderr_tx.send(());
        }

        let exit_code = match child.wait() {
            Ok(status) => status.code().unwrap_or(-1),
            Err(_) => -1,
        };

        debug!("agent — {} exited with code {}", self.name, exit_code);

        let _ = stdout_rx.recv();
        let _ = stderr_rx.recv();

        let stdout = lock_guard(&stdout_buf).clone();
        let stderr = lock_guard(&stderr_buf).clone();

        ExecuteResponse {
            stdout,
            stderr,
            exit_code,
        }
    }
}
