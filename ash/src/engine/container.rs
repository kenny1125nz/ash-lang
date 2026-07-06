use std::io::Write;
use std::process::{Command, Stdio};

use super::config::ContainerConfig;
use super::types::{ExecuteRequest, ExecuteResponse};
use super::Adapter;

pub struct ContainerAdapter {
    name: String,
    config: ContainerConfig,
}

impl ContainerAdapter {
    pub fn new(name: &str, config: ContainerConfig) -> Self {
        ContainerAdapter { name: name.to_string(), config }
    }
}

impl Adapter for ContainerAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, req: &ExecuteRequest) -> ExecuteResponse {
        let mut args = vec!["run".to_string(), "--rm".to_string(), "-i".to_string()];

        if !req.dir.is_empty() {
            args.push("-w".to_string());
            args.push(req.dir.clone());
        }

        for vol in &self.config.volumes {
            args.push("-v".to_string());
            args.push(vol.clone());
        }

        if !req.model.is_empty() {
            args.push("-e".to_string());
            args.push(format!("MODEL={}", req.model));
        }

        args.push(self.config.image.clone());

        let mut child = match Command::new(&self.config.runtime)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                return ExecuteResponse {
                    stdout: String::new(),
                    stderr: format!("failed to spawn {}: {}", self.config.runtime, e),
                    exit_code: -1,
                };
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(req.prompt.as_bytes());
        }

        match child.wait_with_output() {
            Ok(output) => ExecuteResponse {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
            },
            Err(e) => ExecuteResponse {
                stdout: String::new(),
                stderr: format!("failed to wait on container: {}", e),
                exit_code: -1,
            },
        }
    }
}
