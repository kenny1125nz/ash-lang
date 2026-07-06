use std::process::Command;

use super::config::{ApiEndpoint, AuthConfig};
use super::types::{ExecuteRequest, ExecuteResponse};
use super::Adapter;

pub struct ApiAdapter {
    name: String,
    base_url: String,
    _auth: Option<AuthConfig>,
    endpoint: ApiEndpoint,
}

impl ApiAdapter {
    pub fn new(name: &str, base_url: &str, auth: Option<AuthConfig>, endpoint: ApiEndpoint) -> Self {
        ApiAdapter {
            name: name.to_string(),
            base_url: base_url.to_string(),
            _auth: auth,
            endpoint,
        }
    }
}

impl Adapter for ApiAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, req: &ExecuteRequest) -> ExecuteResponse {
        let url = format!("{}{}", self.base_url.trim_end_matches('/'), self.endpoint.path);

        let body = format!(
            r#"{{"prompt":"{}","model":"{}"}}"#,
            req.prompt.replace('"', r#"\""#).replace('\n', r#"\n"#),
            req.model
        );

        match Command::new("curl")
            .arg("-s")
            .arg("-X").arg(&self.endpoint.method)
            .arg("-H").arg("Content-Type: application/json")
            .arg("-d").arg(&body)
            .arg(&url)
            .output()
        {
            Ok(output) => ExecuteResponse {
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                exit_code: output.status.code().unwrap_or(-1),
            },
            Err(e) => ExecuteResponse {
                stdout: String::new(),
                stderr: format!("failed to spawn curl: {}", e),
                exit_code: -1,
            },
        }
    }
}
