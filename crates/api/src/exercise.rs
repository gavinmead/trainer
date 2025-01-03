use crate::TrainerError::{ConnectionError, ExerciseNotFound, UnknownError};
use crate::{TrainerError, TrainerResult};
use async_trait::async_trait;

#[cfg(test)]
use mockall::{automock, predicate::*};

#[derive(Clone, Debug, PartialEq, Copy)]
#[non_exhaustive]
pub enum ExerciseType {
    Barbell,
    KettleBell,
}

impl From<ExerciseType> for i64 {
    fn from(value: ExerciseType) -> Self {
        match value {
            ExerciseType::Barbell => 0,
            ExerciseType::KettleBell => 1,
        }
    }
}

impl From<i64> for ExerciseType {
    fn from(value: i64) -> Self {
        match value {
            0 => ExerciseType::Barbell,
            1 => ExerciseType::KettleBell,
            _ => panic!("unsupported value"),
        }
    }
}

impl From<String> for ExerciseType {
    fn from(value: String) -> Self {
        let lower = value.to_lowercase();
        match lower.as_str() {
            "barbell" => ExerciseType::Barbell,
            "bb" => ExerciseType::Barbell,
            "kettlebell" => ExerciseType::KettleBell,
            "kb" => ExerciseType::KettleBell,
            _ => panic!("unsupported value"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] //this is temporary until code base evolves
pub struct Exercise {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub exercise_type: ExerciseType,
}

#[async_trait]
pub trait ExerciseManagement {
    // Will create or update an exercise
    async fn save(&self, exercise: &mut Exercise) -> TrainerResult<()>;

    async fn get_by_name(&self, name: String) -> TrainerResult<Exercise>;

    async fn get_by_id(&self, id: i64) -> TrainerResult<Exercise>;

    async fn list(&self) -> TrainerResult<Vec<Exercise>>;

    async fn delete(&self, exercise: Exercise) -> TrainerResult<()>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ExerciseRepository {
    /// Persists Exercise
    /// Will return the repository generated ID in a TrainerResult.
    /// TrainerError will be a PersistenceError
    async fn create(&self, exercise: &Exercise) -> TrainerResult<i64>;

    async fn update(&self, exercise: &Exercise) -> TrainerResult<()>;

    async fn query_by_name(&self, name: String) -> TrainerResult<Option<Exercise>>;

    async fn query_by_id(&self, id: i64) -> TrainerResult<Option<Exercise>>;

    async fn list(&self) -> TrainerResult<Vec<Exercise>>;

    async fn delete(&self, id: i64) -> TrainerResult<()>;
}

#[derive(Clone, Debug)]
pub struct ExerciseManager<'a, T: ExerciseRepository> {
    repo: &'a T,
}

impl<'a, T: ExerciseRepository> ExerciseManager<'a, T> {
    #[allow(dead_code)]
    fn new(repo: &'a T) -> TrainerResult<Self> {
        Ok(Self { repo })
    }
}

#[async_trait]
impl<T: ExerciseRepository + Sync> ExerciseManagement for ExerciseManager<'_, T> {
    async fn save(&self, _exercise: &mut Exercise) -> TrainerResult<()> {
        todo!()
    }

    async fn get_by_name(&self, name: String) -> TrainerResult<Exercise> {
        match self.repo.query_by_name(name.clone()).await {
            Ok(o) => match o {
                None => Err(ExerciseNotFound(name.clone())),
                Some(e) => Ok(e),
            },
            Err(err) => {
                match err {
                    TrainerError::ConnectionError(_e) => {
                        //log the backend error message
                        Err(ConnectionError(
                            "error searching repository for exercise".to_string(),
                        ))
                    }
                    _e => Err(UnknownError("unknown error with repository".to_string())),
                }
            }
        }
    }

    async fn get_by_id(&self, _id: i64) -> TrainerResult<Exercise> {
        todo!()
    }

    async fn list(&self) -> TrainerResult<Vec<Exercise>> {
        todo!()
    }

    async fn delete(&self, _exercise: Exercise) -> TrainerResult<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TrainerError;

    #[test]
    fn test_new_ok() {
        let repo = MockExerciseRepository::new();
        let mgr = ExerciseManager::new(&repo);
        assert!(mgr.is_ok())
    }

    #[tokio::test]
    async fn test_get_by_name_ok() {
        let mut repo = MockExerciseRepository::new();
        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .returning(|_string| {
                Ok(Some(Exercise {
                    id: Some(1),
                    name: "Deadlift".to_string(),
                    description: None,
                    exercise_type: ExerciseType::Barbell,
                }))
            });

        let mgr = ExerciseManager::new(&repo).unwrap();

        let get_result = mgr.get_by_name("Deadlift".to_string()).await;
        assert!(get_result.is_ok())
    }

    #[tokio::test]
    async fn test_get_by_name_not_found() {
        let mut repo = MockExerciseRepository::new();
        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .returning(|_string| Ok(None));
        let mgr = ExerciseManager::new(&repo).unwrap();
        let result = mgr.get_by_name("Deadlift".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ExerciseNotFound(s) if s == "Deadlift"
        ));
    }

    #[tokio::test]
    async fn test_get_by_name_repo_sys_error() {
        let mut repo = MockExerciseRepository::new();
        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .returning(|_string| Err(TrainerError::ConnectionError("db_error".to_string())));
        let mgr = ExerciseManager::new(&repo).unwrap();

        let result = mgr.get_by_name("Deadlift".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::ConnectionError(s) if s == "error searching repository for exercise"
        ))
    }
}
