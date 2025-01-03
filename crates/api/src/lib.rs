pub mod exercise;

pub type TrainerResult<T> = Result<T, TrainerError>;

#[derive(thiserror::Error, Debug, Clone)]
#[non_exhaustive]
pub enum TrainerError {
    #[error("ExerciseNotFound: {0}")]
    ExerciseNotFound(String),

    #[error("PersistenceError: {0}")]
    PersistenceError(String),

    #[error("ConnectionError: {0}")]
    ConnectionError(String),

    #[error("QueryError: {0}")]
    QueryError(String),

    #[error("DeleteError: {0}")]
    DeleteError(String),

    #[error("ExerciseIdNotProvidedError: {0}")]
    ExerciseIdNotProvidedError(String),

    #[error("Unknown: {0}")]
    UnknownError(String),
}

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

    #[error("Unknown: {0}")]
    UnknownError(String),
}
