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
        if req.yes {
            if let Some(ref flag) = self.config.yes_flag {
                args.push(flag.clone());
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::config::{ApiEndpoint, ContainerConfig};
    use crate::engine::types::AgentType;

    fn make_config() -> AgentConfig {
        AgentConfig {
            name: "test-agent".into(),
            agent_type: AgentType::LocalCli,
            driver: None,
            cmd: "test-bin".into(),
            args: vec![],
            model_flag: None,
            session_flag: None,
            message_flag: None,
            stdin_prompt: false,
            yes_flag: None,
            base_url: String::new(),
            auth: None,
            endpoint: ApiEndpoint {
                method: String::new(),
                path: String::new(),
            },
            container: ContainerConfig {
                runtime: String::new(),
                image: String::new(),
                mode: String::new(),
                volumes: vec![],
            },
        }
    }

    fn make_req(prompt: &str) -> ExecuteRequest {
        ExecuteRequest {
            prompt: prompt.into(),
            model: String::new(),
            dir: String::new(),
            session: false,
            yes: false,
        }
    }

    #[test]
    fn test_basic_no_flags() {
        let cfg = make_config();
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&make_req("hello"));
        assert_eq!(spec.cmd, "test-bin");
        assert_eq!(spec.args, vec!["hello"]);
        assert!(!spec.stdin_prompt);
    }

    #[test]
    fn test_prefix_args() {
        let mut cfg = make_config();
        cfg.args = vec!["run".into(), "--verbose".into()];
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&make_req("hello"));
        assert_eq!(spec.args, vec!["run", "--verbose", "hello"]);
    }

    #[test]
    fn test_model_flag_with_model() {
        let mut cfg = make_config();
        cfg.model_flag = Some("--model".into());
        let mut req = make_req("hello");
        req.model = "sonnet".into();
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&req);
        assert_eq!(spec.args, vec!["--model", "sonnet", "hello"]);
    }

    #[test]
    fn test_model_flag_omitted_when_model_empty() {
        let mut cfg = make_config();
        cfg.model_flag = Some("--model".into());
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&make_req("hello"));
        assert_eq!(spec.args, vec!["hello"]);
    }

    #[test]
    fn test_session_flag_when_active() {
        let mut cfg = make_config();
        cfg.session_flag = Some("--continue".into());
        let mut req = make_req("hello");
        req.session = true;
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&req);
        assert_eq!(spec.args, vec!["--continue", "hello"]);
    }

    #[test]
    fn test_session_flag_omitted_when_inactive() {
        let mut cfg = make_config();
        cfg.session_flag = Some("--continue".into());
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&make_req("hello"));
        assert_eq!(spec.args, vec!["hello"]);
    }

    #[test]
    fn test_message_flag() {
        let mut cfg = make_config();
        cfg.message_flag = Some("--msg".into());
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&make_req("hello"));
        assert_eq!(spec.args, vec!["--msg", "hello"]);
    }

    #[test]
    fn test_stdin_prompt_true() {
        let mut cfg = make_config();
        cfg.stdin_prompt = true;
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&make_req("hello"));
        assert!(spec.stdin_prompt);
        assert!(spec.args.is_empty());
    }

    #[test]
    fn test_all_flags_together() {
        let mut cfg = make_config();
        cfg.args = vec!["run".into()];
        cfg.model_flag = Some("--model".into());
        cfg.session_flag = Some("--continue".into());
        cfg.message_flag = Some("--msg".into());
        let mut req = make_req("fix the bug");
        req.model = "sonnet".into();
        req.session = true;
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&req);
        assert_eq!(
            spec.args,
            vec!["run", "--model", "sonnet", "--continue", "--msg", "fix the bug"]
        );
    }

    #[test]
    fn test_session_without_message_flag() {
        let mut cfg = make_config();
        cfg.session_flag = Some("--continue".into());
        let mut req = make_req("hello");
        req.session = true;
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&req);
        assert_eq!(spec.args, vec!["--continue", "hello"]);
    }

    #[test]
    fn test_no_flags_no_prefix_args() {
        let cfg = make_config();
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&make_req(""));
        assert_eq!(spec.args, vec![""]);
    }

    #[test]
    fn test_yes_flag_when_enabled() {
        let mut cfg = make_config();
        cfg.yes_flag = Some("--yolo".into());
        let mut req = make_req("hello");
        req.yes = true;
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&req);
        assert_eq!(spec.args, vec!["--yolo", "hello"]);
    }

    #[test]
    fn test_yes_flag_omitted_when_disabled() {
        let mut cfg = make_config();
        cfg.yes_flag = Some("--yolo".into());
        let driver = GenericDriver::new(cfg);
        let spec = driver.build_command(&make_req("hello"));
        assert_eq!(spec.args, vec!["hello"]);
    }
}
