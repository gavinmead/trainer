use crate::TrainerError::{ConnectionError, ExerciseNotFound, LookupError, UnknownError};
use crate::{RepositoryError, RepositoryResult, TrainerError, TrainerResult};
use async_trait::async_trait;
use tracing::{debug, error, info, instrument, span, warn, Level};

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

    async fn list(&self) -> TrainerResult<Vec<Exercise>>;

    async fn delete(&self, name: String) -> TrainerResult<()>;
}

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

    async fn list(&self) -> RepositoryResult<Vec<Exercise>>;

    async fn delete(&self, name: String) -> RepositoryResult<()>;
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
impl<T: ExerciseRepository + Sync + std::fmt::Debug> ExerciseManagement for ExerciseManager<'_, T> {

    async fn save(&self, _exercise: &mut Exercise) -> TrainerResult<()> {
        todo!()
    }

    #[instrument(skip(self), fields(name = name))]
    async fn get_by_name(&self, name: String) -> TrainerResult<Exercise> {
        match self.repo.query_by_name(name.clone()).await {
            Ok(e) => {
                debug!("exercise found");
                Ok(e)
            },
            Err(err) => {
                match err {
                    RepositoryError::ConnectionError(e) => {
                        //log the backend error message
                        error!("{}", e);
                        Err(LookupError(
                            "error searching repository for exercise".to_string(),
                        ))
                    }
                    RepositoryError::ItemNotFoundError => {
                        debug!("exercise not found");
                        Err(ExerciseNotFound(name.clone()))
                    }
                    err => {
                        error!("{}", err.to_string());
                        Err(LookupError("unknown error with repository".to_string()))
                    },
                }
            }
        }
    }

    async fn list(&self) -> TrainerResult<Vec<Exercise>> {
        todo!()
    }

    async fn delete(&self, _name: String) -> TrainerResult<()> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::RepositoryError::ItemNotFoundError;
    use super::*;
    use crate::{RepositoryError, TrainerError};
    use test_log::test;

    #[test]
    fn test_new_ok() {
        let repo = MockExerciseRepository::new();
        let mgr = ExerciseManager::new(&repo);
        assert!(mgr.is_ok())
    }


    #[test(tokio::test)]
    async fn test_get_by_name_ok() {

        let mut repo = MockExerciseRepository::new();
        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .returning(|_string| {
                Ok(Exercise {
                    id: Some(1),
                    name: "Deadlift".to_string(),
                    description: None,
                    exercise_type: ExerciseType::Barbell,
                })
            });

        let mgr = ExerciseManager::new(&repo).unwrap();

        let get_result = mgr.get_by_name("Deadlift".to_string()).await;
        assert!(get_result.is_ok())
    }

    #[test(tokio::test)]
    async fn test_get_by_name_not_found() {
        let mut repo = MockExerciseRepository::new();
        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .returning(|_string| Err(ItemNotFoundError));
        let mgr = ExerciseManager::new(&repo).unwrap();
        let result = mgr.get_by_name("Deadlift".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ExerciseNotFound(s) if s == "Deadlift"
        ));
    }

    #[test(tokio::test)]
    async fn test_get_by_name_repo_sys_error() {
        let mut repo = MockExerciseRepository::new();
        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .returning(|_string| Err(RepositoryError::ConnectionError("db_error".to_string())));
        let mgr = ExerciseManager::new(&repo).unwrap();

        let result = mgr.get_by_name("Deadlift".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::LookupError(s) if s == "error searching repository for exercise"
        ))
    }

    #[test(tokio::test)]
    async fn test_get_by_name_unknown_repo_error() {
        let mut repo = MockExerciseRepository::new();
        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .returning(|_string| Err(RepositoryError::UnknownError("db_error".to_string())));
        let mgr = ExerciseManager::new(&repo).unwrap();

        let result = mgr.get_by_name("Deadlift".to_string()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::LookupError(s) if s == "unknown error with repository"
        ))
    }

    #[test]
    fn from_string_to_exercise_type_ok() {
        let bbs = vec![
            "Barbell".to_string(),
            "BARBELL".to_string(),
            "bArBeLl".to_string(),
            "bb".to_string(),
            "BB".to_string(),
            "bB".to_string(),
        ];
        let kbs = vec![
            "Kettlebell".to_string(),
            "KETTLEBELL".to_string(),
            "kEtTlEbElL".to_string(),
            "kb".to_string(),
            "KB".to_string(),
            "kB".to_string(),
        ];

        for bb in bbs {
            let et: ExerciseType = bb.into();
            assert_eq!(et, ExerciseType::Barbell)
        }

        for kb in kbs {
            let et: ExerciseType = kb.into();
            assert_eq!(et, ExerciseType::KettleBell)
        }
    }

    #[test]
    #[should_panic]
    fn from_string_to_exercise_type_fail() {
        let _: ExerciseType = "not_found".to_string().into();
    }

    #[test]
    #[should_panic]
    fn test_bad_i64_for_exercise_type() {
        let _ = ExerciseType::from(1000);
    }
}
