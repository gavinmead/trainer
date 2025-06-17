pub type ExerciseResult<T, E = ExerciseError> = Result<T, E>;
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ExerciseError {
    ExerciseNotFoundError,
    LookupError,
    SaveFailed,
    DeleteFailed,
    UnknownError,
}

pub type RepositoryResult<T, E = RepositoryError> = Result<T, E>;

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
