use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

use crate::Exercise;
use crate::RepositoryResult;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ExerciseRepository {
    /// Persists Exercise
    /// Will return the repository generated ID in a TrainerResult.
    /// RepositoryError will be a PersistenceError
    async fn create(&self, exercise: &Exercise) -> RepositoryResult<i64>;

    async fn update(&self, exercise: &Exercise) -> RepositoryResult<()>;

    // Retrieves the exercise by its unique name.
    // Will return an ItemNotFoundError if the item does not exist
    async fn query_by_name(&self, name: String) -> RepositoryResult<Exercise>;

    // Will return an ItemNotFoundError if the item does not exist
    async fn query_by_id(&self, id: i64) -> RepositoryResult<Exercise>;

    async fn list(&self) -> RepositoryResult<Vec<Exercise>>;

    /// Deletes an exercise from the repository
    async fn delete(&self, id: i64) -> RepositoryResult<()>;
}
