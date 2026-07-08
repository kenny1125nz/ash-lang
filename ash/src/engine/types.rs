#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum AgentType {
    LocalCli,
    Api,
    Container,
    Browser,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::LocalCli => "local-cli",
            AgentType::Api => "api",
            AgentType::Container => "container",
            AgentType::Browser => "browser",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecuteRequest {
    pub prompt: String,
    pub model: String,
    pub dir: String,
    pub session: bool,
    pub yes: bool,
}

#[derive(Debug, Clone)]
pub struct ExecuteResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}
