use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("forbidden")]
    Forbidden,
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("internal error")]
    Internal,
}

pub type DomainResult<T> = Result<T, DomainError>;
