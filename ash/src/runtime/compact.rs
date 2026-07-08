#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub mode: String,
    pub window: String,
    pub strategy: String,
}

impl Config {
    pub fn new() -> Self {
        Config {
            mode: "auto".to_string(),
            window: "32000".to_string(),
            strategy: "truncate".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    pub action: String,
    pub args: Vec<String>,
}

use crate::AshError;

impl Directive {
    pub fn parse(s: &str) -> Result<Directive, AshError> {
        let s = s.trim();
        if s.is_empty() {
            return Err(AshError::Msg("empty compact directive".to_string()));
        }
        let parts: Vec<String> = s.split_whitespace().map(|p| p.to_string()).collect();
        if parts.is_empty() {
            return Err(AshError::Msg("empty compact directive".to_string()));
        }
        let action = parts[0].clone();
        let args = parts[1..].to_vec();
        Ok(Directive { action, args })
    }

    pub fn apply(&self, config: &mut Config) {
        match self.action.as_str() {
            "on" => config.mode = "on".to_string(),
            "off" => config.mode = "off".to_string(),
            "auto" => config.mode = "auto".to_string(),
            "truncate" => {
                config.strategy = "truncate".to_string();
                if let Some(w) = self.args.first() {
                    if w.parse::<usize>().is_ok() {
                        config.window = w.clone();
                    }
                }
            }
            "summarize" => {
                config.strategy = "summarize".to_string();
            }
            "window" => {
                config.strategy = "window".to_string();
                if let Some(w) = self.args.first() {
                    if w.parse::<usize>().is_ok() {
                        config.window = w.clone();
                    }
                }
            }
            "drop" => {
                config.strategy = "drop".to_string();
            }
            _ => {}
        }
    }
}
