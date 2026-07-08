use thiserror::Error;

use crate::eval::EvalError;

#[derive(Error, Debug)]
pub enum AshError {
    #[error("parse error: {0}")]
    Parse(String),
    #[error("eval error: {0}")]
    Eval(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Msg(String),
}

impl From<EvalError> for AshError {
    fn from(e: EvalError) -> Self {
        match e {
            EvalError::Exit(ex) => AshError::Msg(format!("exit code {}", ex.code)),
            EvalError::Msg(s) => AshError::Eval(s),
        }
    }
}
