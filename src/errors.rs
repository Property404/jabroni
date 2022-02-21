//! Errors for use within this crate.

/// The error type used ubiquitously within this crate.
#[derive(thiserror::Error, Debug)]
pub enum JabroniError {
    /// Problem parsing.
    #[error("ParseError: {0}")]
    Parse(String),
    /// Type mismatch.
    #[error("TypeError: {0}")]
    Type(String),
    /// Binding doesn't exist
    #[error("ReferenceError: {0}")]
    Reference(String),
    /// Defining a variable or constant twice.
    #[error("DoubleDefinitionError: {0}")]
    DoubleDefinition(String),
}

/// The result type used ubiquitously within this crate.
pub type JabroniResult<T = ()> = Result<T, JabroniError>;
