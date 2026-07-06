use super::config::AgentConfig;
use super::types::ExecuteRequest;

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub cmd: String,
    pub args: Vec<String>,
    pub stdin_prompt: bool,
}

pub trait LocalCliDriver: Send + Sync {
    fn name(&self) -> &str;
    fn build_command(&self, req: &ExecuteRequest) -> CommandSpec;
}

pub struct OpenCodeDriver;

impl LocalCliDriver for OpenCodeDriver {
    fn name(&self) -> &str {
        "opencode"
    }

    fn build_command(&self, req: &ExecuteRequest) -> CommandSpec {
        let mut args = vec!["run".to_string()];
        if req.session {
            args.push("--continue".to_string());
        }
        if !req.model.is_empty() {
            args.push("--model".to_string());
            args.push(req.model.clone());
        }
        args.push(req.prompt.clone());
        CommandSpec {
            cmd: "opencode".into(),
            args,
            stdin_prompt: false,
        }
    }
}

pub struct ClaudeDriver;

impl LocalCliDriver for ClaudeDriver {
    fn name(&self) -> &str {
        "claude-code"
    }

    fn build_command(&self, req: &ExecuteRequest) -> CommandSpec {
        let mut args = Vec::new();
        if req.session {
            args.push("--continue".to_string());
        }
        if !req.model.is_empty() {
            args.push("--model".to_string());
            args.push(req.model.clone());
        }
        args.push("--msg".to_string());
        args.push(req.prompt.clone());
        CommandSpec {
            cmd: "claude".into(),
            args,
            stdin_prompt: false,
        }
    }
}

pub struct AiderDriver;

impl LocalCliDriver for AiderDriver {
    fn name(&self) -> &str {
        "aider"
    }

    fn build_command(&self, req: &ExecuteRequest) -> CommandSpec {
        let mut args = Vec::new();
        if !req.model.is_empty() {
            args.push("--model".to_string());
            args.push(req.model.clone());
        }
        if req.session {
            args.push("--restore-chat-history".to_string());
        }
        args.push("--msg".to_string());
        args.push(req.prompt.clone());
        CommandSpec {
            cmd: "aider".into(),
            args,
            stdin_prompt: false,
        }
    }
}

pub struct EchoDriver;

impl LocalCliDriver for EchoDriver {
    fn name(&self) -> &str {
        "echo"
    }

    fn build_command(&self, req: &ExecuteRequest) -> CommandSpec {
        CommandSpec {
            cmd: "echo".into(),
            args: vec![req.prompt.clone()],
            stdin_prompt: false,
        }
    }
}

/// A driver that reads all command construction from an `AgentConfig`.
///
/// Supports `model_flag`, `session_flag`, `message_flag`, and `stdin_prompt`
/// fields so that any CLI agent can be registered without writing new Rust code.
pub struct GenericDriver {
    pub config: AgentConfig,
}

impl GenericDriver {
    pub fn new(config: AgentConfig) -> Self {
        GenericDriver { config }
    }
}

impl LocalCliDriver for GenericDriver {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn build_command(&self, req: &ExecuteRequest) -> CommandSpec {
        let mut args = self.config.args.clone();
        if !req.model.is_empty() {
            if let Some(ref flag) = self.config.model_flag {
                args.push(flag.clone());
                args.push(req.model.clone());
            }
        }
        if req.session {
            if let Some(ref flag) = self.config.session_flag {
                args.push(flag.clone());
            }
        }
        if let Some(ref flag) = self.config.message_flag {
            args.push(flag.clone());
            args.push(req.prompt.clone());
        } else if !self.config.stdin_prompt {
            args.push(req.prompt.clone());
        }
        CommandSpec {
            cmd: self.config.cmd.clone(),
            args,
            stdin_prompt: self.config.stdin_prompt,
        }
    }
}
