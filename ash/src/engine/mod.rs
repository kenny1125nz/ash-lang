pub mod adapter;
pub mod api;
pub mod browser;
pub mod config;
pub mod container;
pub mod discovery;
pub mod driver;
pub mod types;

pub use adapter::{Adapter, LocalCliAdapter};
pub use api::ApiAdapter;
pub use browser::{BrowserAdapter, BrowserFallback};
pub use config::{AgentConfig, ApiEndpoint, AuthConfig, ContainerConfig};
pub use container::ContainerAdapter;
pub use discovery::{discover, discover_and_register, discovery_summary, generate_yaml, print_discovery, read_config, write_config, DiscoveryResult};
pub use driver::{AiderDriver, ClaudeDriver, CommandSpec, EchoDriver, GenericDriver, LocalCliDriver, OpenCodeDriver};
pub use types::{AgentType, ExecuteRequest, ExecuteResponse};

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

fn registry() -> &'static Mutex<HashMap<String, Arc<dyn Adapter>>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, Arc<dyn Adapter>>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn register(name: &str, adapter: Arc<dyn Adapter>) {
    registry().lock().unwrap().insert(name.to_string(), adapter);
}

pub fn get(name: &str) -> Option<Arc<dyn Adapter>> {
    registry().lock().unwrap().get(name).cloned()
}

/// Build an adapter from an `AgentConfig`.
pub fn from_config(cfg: &AgentConfig) -> Arc<dyn Adapter> {
    match cfg.agent_type {
        AgentType::LocalCli => {
            let driver: Arc<dyn LocalCliDriver> = match cfg.driver.as_deref() {
                Some("echo") => Arc::new(EchoDriver),
                Some("opencode") => Arc::new(OpenCodeDriver),
                Some("claude-code") | Some("claude") => Arc::new(ClaudeDriver),
                Some("aider") => Arc::new(AiderDriver),
                // Named but unknown driver, or no driver field — use the generic path
                _ => Arc::new(GenericDriver::new(cfg.clone())),
            };
            Arc::new(LocalCliAdapter::new(&cfg.name, driver))
        }
        AgentType::Api => Arc::new(ApiAdapter::new(
            &cfg.name,
            &cfg.base_url,
            cfg.auth.clone(),
            cfg.endpoint.clone(),
        )),
        AgentType::Container => Arc::new(ContainerAdapter::new(&cfg.name, cfg.container.clone())),
        AgentType::Browser => Arc::new(BrowserFallback::new(&cfg.name)),
    }
}

pub fn register_defaults() {
    register(
        "echo",
        Arc::new(LocalCliAdapter::new("echo", Arc::new(EchoDriver))),
    );

    register(
        "opencode",
        Arc::new(LocalCliAdapter::new("opencode", Arc::new(OpenCodeDriver))),
    );

    register(
        "claude-code",
        Arc::new(LocalCliAdapter::new("claude-code", Arc::new(ClaudeDriver))),
    );

    register(
        "aider",
        Arc::new(LocalCliAdapter::new("aider", Arc::new(AiderDriver))),
    );

}
