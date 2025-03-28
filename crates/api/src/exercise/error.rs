pub type ExerciseResult<T> = std::result::Result<T, ExerciseError>;
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ExerciseError {
    ExerciseNotFoundError,
    LookupError,
    SaveFailed,
    DeleteFailed,
    UnknownError,
}

pub type RepositoryResult<T> = Result<T, crate::RepositoryError>;

#[derive(thiserror::Error, Debug, Clone)]
#[non_exhaustive]
pub enum RepositoryError {
    #[error("PersistenceError: {0}")]
    PersistenceError(String),

    #[error("ConnectionError: {0}")]
    ConnectionError(String),

    #[error("QueryError: {0}")]
    QueryError(String),

    #[error("DeleteError: {0}")]
    DeleteError(String),

    #[error("ItemNotFoundError")]
    ItemNotFoundError,

    #[error("DuplicateIdError")]
    DuplicateIdError,

    #[error("Unknown: {0}")]
    UnknownError(String),
}
