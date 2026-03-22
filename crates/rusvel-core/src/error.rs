use thiserror::Error;

/// Central error type for all RUSVEL operations.
///
/// Adapters and engines map their internal errors into these variants.
/// The `Internal` variant carries an opaque message for unexpected failures.
#[derive(Error, Debug)]
pub enum RusvelError {
    // ── Lookup / CRUD ──────────────────────────────────────────────
    #[error("not found: {kind} `{id}`")]
    NotFound { kind: String, id: String },

    #[error("already exists: {kind} `{id}`")]
    AlreadyExists { kind: String, id: String },

    // ── Validation ─────────────────────────────────────────────────
    #[error("validation: {0}")]
    Validation(String),

    #[error("invalid state transition from `{from}` to `{to}`")]
    InvalidState { from: String, to: String },

    // ── Auth / Credentials ─────────────────────────────────────────
    #[error("unauthorized: {0}")]
    Unauthorized(String),

    // ── LLM / Agent ────────────────────────────────────────────────
    #[error("llm error: {0}")]
    Llm(String),

    #[error("agent error: {0}")]
    Agent(String),

    #[error("tool error: {0}")]
    Tool(String),

    #[error("budget exceeded: spent {spent}, limit {limit}")]
    BudgetExceeded { spent: f64, limit: f64 },

    // ── Storage / IO ───────────────────────────────────────────────
    #[error("storage error: {0}")]
    Storage(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    // ── Config ─────────────────────────────────────────────────────
    #[error("config error: {0}")]
    Config(String),

    // ── Catch-all ──────────────────────────────────────────────────
    #[error("internal: {0}")]
    Internal(String),
}

/// Crate-level result alias used throughout `rusvel-core` and by all ports.
pub type Result<T> = std::result::Result<T, RusvelError>;

// ── Convenience conversions ────────────────────────────────────────

impl From<serde_json::Error> for RusvelError {
    fn from(e: serde_json::Error) -> Self {
        RusvelError::Serialization(e.to_string())
    }
}
