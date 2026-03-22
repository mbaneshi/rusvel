//! Code engine error types.

/// Errors specific to the code engine.
#[derive(Debug, thiserror::Error)]
pub enum CodeError {
    #[error("io: {0}")]
    Io(String),
    #[error("parse: {0}")]
    Parse(String),
    #[error("internal: {0}")]
    Internal(String),
}

/// Code engine result alias.
pub type Result<T> = std::result::Result<T, CodeError>;

impl From<CodeError> for rusvel_core::error::RusvelError {
    fn from(e: CodeError) -> Self {
        rusvel_core::error::RusvelError::Internal(e.to_string())
    }
}
