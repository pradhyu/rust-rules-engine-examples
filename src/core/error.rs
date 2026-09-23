use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum EngineError {
    #[error("Fact not found at path: {0}")]
    FactNotFound(String),

    #[error("Invalid type at path: {path}. Expected {expected}, got {actual}")]
    TypeMismatch {
        path: String,
        expected: String,
        actual: String,
    },

    #[error("Condition evaluation failed: {0}")]
    ConditionError(String),

    #[error("Invalid rule definition in rule '{rule_id}': {message}")]
    InvalidRuleDefinition {
        rule_id: String,
        message: String,
    },

    #[error("Matrix lookup failed: {0}")]
    MatrixLookupError(String),

    #[error("Serialization / Deserialization error: {0}")]
    SerializationError(String),

    #[error("General evaluation error: {0}")]
    EvaluationError(String),
}

pub type Result<T> = std::result::Result<T, EngineError>;
