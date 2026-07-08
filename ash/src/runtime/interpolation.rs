use crate::AshError;
use crate::lang::ast::{InterpSpan, InterpType};
use crate::runtime::value::Value;
use regex::Regex;

pub struct Interpolation;

impl Interpolation {
    pub fn resolve(
        s: &str,
        get_var: impl Fn(&str) -> Option<String>,
        exec_cmd: impl Fn(&str) -> Result<String, AshError>,
    ) -> Result<String, AshError> {
        let mut s = s.to_string();

        let mut idx = s.find("\\$");
        while let Some(pos) = idx {
            s.replace_range(pos..pos + 2, "\x00");
            idx = s.find("\\$");
        }

        let re = Regex::new(r"\$\{([^}]+)\}|\$\(([^)]+)\)").unwrap();
        let result = re.replace_all(&s, |caps: &regex::Captures| {
            if let Some(var) = caps.get(1) {
                let name = var.as_str();
                match get_var(name) {
                    Some(val) => val,
                    None => format!("${{{}}}", name),
                }
            } else if let Some(cmd) = caps.get(2) {
                let cmd_str = cmd.as_str();
                match exec_cmd(cmd_str) {
                    Ok(out) => out,
                    Err(_) => String::new(),
                }
            } else {
                caps.get(0).unwrap().as_str().to_string()
            }
        });

        let result = result.replace("\x00", "$");
        Ok(result)
    }

    pub fn resolve_spans(
        interps: &[InterpSpan],
        get_var: &dyn Fn(&str) -> Option<Value>,
        exec_cmd: &dyn Fn(&str) -> Result<String, AshError>,
    ) -> Result<String, AshError> {
        let mut result = String::new();
        for span in interps {
            match &span.typ {
                InterpType::Var(name) => {
                    match get_var(name) {
                        Some(val) => result.push_str(&format!("{}", val)),
                        None => {
                            return Err(AshError::Msg(format!("undefined variable: {}", name)));
                        }
                    }
                }
                InterpType::Cmd(cmd) => {
                    let out = exec_cmd(cmd)?;
                    result.push_str(&out);
                }
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_var(_: &str) -> Option<String> {
        None
    }

    fn no_cmd(_: &str) -> Result<String, AshError> {
        Err(AshError::Msg("no command execution".to_string()))
    }

    fn test_var(name: &str) -> Option<String> {
        match name {
            "NAME" => Some("world".to_string()),
            "X" => Some("42".to_string()),
            _ => None,
        }
    }

    fn test_cmd(cmd: &str) -> Result<String, AshError> {
        match cmd {
            "echo hello" => Ok("hello\n".to_string()),
            _ => Err(AshError::Msg(format!("unknown command: {}", cmd))),
        }
    }

    #[test]
    fn test_interpolation_empty() {
        let result = Interpolation::resolve("", no_var, no_cmd).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_interpolation_no_match() {
        let result = Interpolation::resolve("hello world", no_var, no_cmd).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_interpolation_var() {
        let result = Interpolation::resolve("hello ${NAME}", test_var, no_cmd).unwrap();
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_interpolation_undefined_var() {
        let result = Interpolation::resolve("hello ${UNDEFINED}", test_var, no_cmd).unwrap();
        assert_eq!(result, "hello ${UNDEFINED}");
    }

    #[test]
    fn test_interpolation_multi() {
        let result =
            Interpolation::resolve("${X} + ${Y} = ?", test_var, no_cmd).unwrap();
        assert_eq!(result, "42 + ${Y} = ?");
    }

    #[test]
    fn test_interpolation_cmd() {
        let result =
            Interpolation::resolve("result: $(echo hello)", no_var, test_cmd).unwrap();
        assert_eq!(result, "result: hello\n");
    }

    #[test]
    fn test_interpolation_escape() {
        let result =
            Interpolation::resolve(r"hello \${NAME}", test_var, no_cmd).unwrap();
        assert_eq!(result, "hello ${NAME}");
    }

    #[test]
    fn test_resolve_spans() {
        use crate::lang::ast::Pos;
        let interps = vec![
            InterpSpan {
                pos: Pos { line: 1, col: 1 },
                typ: InterpType::Var("NAME".to_string()),
            },
        ];
        let get_var = |s: &str| -> Option<Value> {
            match s {
                "NAME" => Some(Value::String("world".to_string())),
                _ => None,
            }
        };
        let result = Interpolation::resolve_spans(&interps, &get_var, &no_cmd).unwrap();
        assert_eq!(result, "world");
    }

    #[test]
    fn test_resolve_spans_undefined() {
        use crate::lang::ast::Pos;
        let interps = vec![
            InterpSpan {
                pos: Pos { line: 1, col: 1 },
                typ: InterpType::Var("MISSING".to_string()),
            },
        ];
        let get_var = |s: &str| -> Option<Value> {
            match s {
                "NAME" => Some(Value::String("world".to_string())),
                _ => None,
            }
        };
        let result = Interpolation::resolve_spans(&interps, &get_var, &no_cmd);
        assert!(result.is_err());
    }
}
