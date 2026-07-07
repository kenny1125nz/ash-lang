use regex::Regex;

use super::event::TelemetryEvent;

pub struct Filter {
    topic_include: Vec<Regex>,
    topic_exclude: Vec<Regex>,
    agent_include: Vec<Regex>,
    agent_exclude: Vec<Regex>,
    model_include: Vec<Regex>,
    model_exclude: Vec<Regex>,
    pub capture_payload: bool,
}

impl Filter {
    pub fn new(rules: Option<&str>) -> Self {
        let mut topic_include = Vec::new();
        let mut topic_exclude = Vec::new();
        let mut agent_include = Vec::new();
        let mut agent_exclude = Vec::new();
        let mut model_include = Vec::new();
        let mut model_exclude = Vec::new();
        let mut capture_payload = false;

        if let Some(s) = rules {
            for token in s.split(',') {
                let token = token.trim();
                if token.is_empty() {
                    continue;
                }
                if let Some(rest) = token.strip_prefix("topic=") {
                    topic_include.push(pat_to_regex(rest));
                } else if let Some(rest) = token.strip_prefix("topic!=") {
                    topic_exclude.push(pat_to_regex(rest));
                } else if let Some(rest) = token.strip_prefix("agent=") {
                    agent_include.push(pat_to_regex(rest));
                } else if let Some(rest) = token.strip_prefix("agent!=") {
                    agent_exclude.push(pat_to_regex(rest));
                } else if let Some(rest) = token.strip_prefix("model=") {
                    model_include.push(pat_to_regex(rest));
                } else if let Some(rest) = token.strip_prefix("model!=") {
                    model_exclude.push(pat_to_regex(rest));
                } else if token == "payload:true" {
                    capture_payload = true;
                } else if token == "payload:false" {
                    capture_payload = false;
                }
            }
        }

        Filter {
            topic_include,
            topic_exclude,
            agent_include,
            agent_exclude,
            model_include,
            model_exclude,
            capture_payload,
        }
    }

    pub fn accept(&self, event: &TelemetryEvent) -> bool {
        let topic = event.kind.as_str();

        if matches_any(topic, &self.topic_exclude) {
            return false;
        }
        if !self.topic_include.is_empty() && !matches_any(topic, &self.topic_include) {
            return false;
        }

        if let Some(agent) = event.payload.get("agent").and_then(|v| v.as_str()) {
            if !agent.is_empty() {
                if matches_any(agent, &self.agent_exclude) {
                    return false;
                }
                if !self.agent_include.is_empty() && !matches_any(agent, &self.agent_include) {
                    return false;
                }
            }
        }

        if let Some(model) = event.payload.get("model").and_then(|v| v.as_str()) {
            if !model.is_empty() {
                if matches_any(model, &self.model_exclude) {
                    return false;
                }
                if !self.model_include.is_empty() && !matches_any(model, &self.model_include) {
                    return false;
                }
            }
        }

        true
    }
}

fn pat_to_regex(pat: &str) -> Regex {
    let mut re = String::with_capacity(pat.len() + 4);
    re.push('^');
    for ch in pat.chars() {
        match ch {
            '*' => re.push_str(".*"),
            '.' => re.push_str("\\."),
            c => re.push(c),
        }
    }
    re.push('$');
    Regex::new(&re).unwrap_or_else(|_| Regex::new("^$").unwrap())
}

fn matches_any(s: &str, patterns: &[Regex]) -> bool {
    patterns.iter().any(|r| r.is_match(s))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::context::SpanContext;
    use crate::telemetry::event::EventKind;

    fn event(kind: EventKind, agent: &str, model: &str) -> TelemetryEvent {
        TelemetryEvent {
            ctx: SpanContext::root(),
            timestamp: 0,
            kind,
            payload: serde_json::json!({"agent": agent, "model": model}),
        }
    }

    fn event_no_agent(kind: EventKind) -> TelemetryEvent {
        TelemetryEvent {
            ctx: SpanContext::root(),
            timestamp: 0,
            kind,
            payload: serde_json::Value::Null,
        }
    }

    #[test]
    fn test_empty_rules_pass_all() {
        let f = Filter::new(None);
        assert!(f.accept(&event(EventKind::SessionStart, "", "")));
        assert!(f.accept(&event(EventKind::AgentCall, "echo", "")));
    }

    #[test]
    fn test_topic_include() {
        let f = Filter::new(Some("topic=agent*"));
        assert!(f.accept(&event(EventKind::AgentCall, "", "")));
        assert!(f.accept(&event(EventKind::AgentResponse, "", "")));
        assert!(!f.accept(&event(EventKind::SessionStart, "", "")));
        assert!(!f.accept(&event(EventKind::CommandExec, "", "")));
    }

    #[test]
    fn test_topic_exclude() {
        let f = Filter::new(Some("topic!=session_*"));
        assert!(f.accept(&event(EventKind::AgentCall, "", "")));
        assert!(!f.accept(&event(EventKind::SessionStart, "", "")));
        assert!(!f.accept(&event(EventKind::SessionEnd, "", "")));
    }

    #[test]
    fn test_agent_include() {
        let f = Filter::new(Some("agent=opencode"));
        assert!(f.accept(&event(EventKind::AgentCall, "opencode", "")));
        assert!(!f.accept(&event(EventKind::AgentCall, "echo", "")));
        // agent-less events (no agent field) should pass
        assert!(f.accept(&event_no_agent(EventKind::SessionStart)));
    }

    #[test]
    fn test_agent_exclude() {
        let f = Filter::new(Some("agent!=echo"));
        assert!(!f.accept(&event(EventKind::AgentCall, "echo", "")));
        assert!(f.accept(&event(EventKind::AgentCall, "opencode", "")));
    }

    #[test]
    fn test_model_include() {
        let f = Filter::new(Some("model=*sonnet*"));
        assert!(f.accept(&event(EventKind::AgentCall, "opencode", "claude-sonnet-4")));
        assert!(!f.accept(&event(EventKind::AgentCall, "opencode", "gpt-4")));
    }

    #[test]
    fn test_model_exclude() {
        let f = Filter::new(Some("model!=deepseek*"));
        assert!(!f.accept(&event(EventKind::AgentCall, "opencode", "deepseek-v4")));
        assert!(f.accept(&event(EventKind::AgentCall, "opencode", "gpt-4")));
    }

    #[test]
    fn test_capture_payload() {
        let f = Filter::new(Some("payload:true"));
        assert!(f.capture_payload);

        let f = Filter::new(Some("payload:false"));
        assert!(!f.capture_payload);

        let f = Filter::new(None);
        assert!(!f.capture_payload);
    }

    #[test]
    fn test_multiple_rules_anded() {
        let f = Filter::new(Some("topic=agent*,agent=opencode"));
        assert!(f.accept(&event(EventKind::AgentCall, "opencode", "")));
        assert!(!f.accept(&event(EventKind::AgentCall, "echo", "")));   // agent reject
        assert!(!f.accept(&event(EventKind::SessionStart, "", "")));    // topic reject
    }

    #[test]
    fn test_wildcard_patterns() {
        let f = Filter::new(Some("topic=agent*"));
        assert!(f.accept(&event(EventKind::AgentCall, "", "")));
        assert!(f.accept(&event(EventKind::AgentResponse, "", "")));
        assert!(!f.accept(&event(EventKind::CommandExec, "", "")));
        assert!(!f.accept(&event(EventKind::Error, "", "")));
    }
}
