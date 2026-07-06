use std::sync::Arc;

use super::types::{ExecuteRequest, ExecuteResponse};
use super::Adapter;

pub type BrowserHandler = Arc<dyn Fn(&ExecuteRequest) -> ExecuteResponse + Send + Sync>;

/// Adapter for in-browser JS agents when Ash is compiled to WebAssembly.
///
/// Instead of shelling out, this delegates to a callback supplied by the JS host
/// (e.g. a lightweight JS-based agent running in a browser playground).
/// On native builds, no handler is registered by default — an error adapter is used instead.
pub struct BrowserAdapter {
    name: String,
    handler: BrowserHandler,
}

impl BrowserAdapter {
    pub fn new(name: &str, handler: BrowserHandler) -> Self {
        BrowserAdapter {
            name: name.to_string(),
            handler,
        }
    }
}

impl Adapter for BrowserAdapter {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, req: &ExecuteRequest) -> ExecuteResponse {
        (self.handler)(req)
    }
}

/// Fallback used when a config references a browser agent but no JS handler is registered.
pub struct BrowserFallback {
    name: String,
}

impl BrowserFallback {
    pub fn new(name: &str) -> Self {
        BrowserFallback { name: name.to_string() }
    }
}

impl Adapter for BrowserFallback {
    fn name(&self) -> &str {
        &self.name
    }

    fn execute(&self, _req: &ExecuteRequest) -> ExecuteResponse {
        ExecuteResponse {
            stdout: String::new(),
            stderr: format!(
                "browser agent '{}' requires a JS host to register a handler \
                 (Ash must be compiled to WebAssembly)",
                self.name
            ),
            exit_code: -1,
        }
    }
}
