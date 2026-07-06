use super::types::AgentType;

/// Auth configuration for API adapters.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// "bearer", "basic", or "header"
    pub auth_type: String,
    /// Name of the environment variable holding the token
    pub token_env: String,
}

/// Endpoint configuration for API adapters.
#[derive(Debug, Clone)]
pub struct ApiEndpoint {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// URL path relative to base_url
    pub path: String,
}

/// Container runtime configuration.
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// "docker" or "k8s"
    pub runtime: String,
    /// Container image
    pub image: String,
    /// "run" (create+destroy) or "exec" (attach to existing)
    pub mode: String,
    /// Volume mounts: "host:container" pairs
    pub volumes: Vec<String>,
}

/// Complete agent configuration for any adapter type.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub agent_type: AgentType,
    /// Local CLI fields
    pub driver: Option<String>,
    pub cmd: String,
    pub args: Vec<String>,
    /// Flag passed before the model value, e.g. "--model". Omitted when model is empty.
    pub model_flag: Option<String>,
    /// Flag passed when session=true, e.g. "--continue". Omitted when session=false.
    pub session_flag: Option<String>,
    /// Flag passed before the prompt, e.g. "--msg". When None, prompt is the last positional arg.
    pub message_flag: Option<String>,
    /// When true, the prompt is piped to stdin instead of passed as a CLI arg.
    pub stdin_prompt: bool,
    /// API adapter fields
    pub base_url: String,
    pub auth: Option<AuthConfig>,
    pub endpoint: ApiEndpoint,
    /// Container adapter fields
    pub container: ContainerConfig,
}
