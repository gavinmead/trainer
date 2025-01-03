use crate::TrainerError::{ExerciseNotFound, LookupError};
use crate::{RepositoryError, RepositoryResult, TrainerError, TrainerResult};
use async_trait::async_trait;
use tracing::{debug, error, instrument};

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

    // Will return an ItemNotFoundError if the item does not exist
    async fn query_by_id(&self, id: i64) -> RepositoryResult<Exercise>;

    async fn list(&self) -> RepositoryResult<Vec<Exercise>>;

    async fn delete(&self, id: i64) -> RepositoryResult<()>;
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

    async fn process_save(&self, exercise: &mut Exercise) -> TrainerResult<()> {
        let create_result = self.repo.create(exercise).await;
        match create_result {
            Ok(id) => {
                debug!("received id {} from repository", &id);
                exercise.id = Some(id);
                Ok(())
            }
            Err(err) => match err {
                RepositoryError::PersistenceError(err) => {
                    error!("{}", err);
                    Err(TrainerError::PersistenceError(
                        "an error occurred while creating a new exercise".to_string(),
                    ))
                }
                e => {
                    error!("{}", e.to_string());
                    Err(TrainerError::UnknownError(
                        "an unknown error occurred while creating a new exercise".to_string(),
                    ))
                }
            },
        }
    }
}

#[async_trait]
impl<T: ExerciseRepository + Sync + std::fmt::Debug> ExerciseManagement for ExerciseManager<'_, T> {
    //! save creates or updates an existing exercise.
    //! # Arguments
    //! * `exercise` - the exercise to save.  This is mutable so the manager can assign a unique internal
    //! identifier
    //!
    //! # Returns
    //! * [`Ok`]` if the save is successful
    //! * A [`TrainerError::PersistenceError`] if there is a problem saving the exercise with
    //! the [`T`] repository implementation
    //! * A [`TrainerError::ExerciseNotFound`] if the internal identifier associated with the
    //! exercise is not found in the repository
    //! * A [`TrainerError::UnknownError`] if there is some other problem saving the exercise
    #[instrument(skip(self), fields(name = exercise.name))]
    async fn save(&self, exercise: &mut Exercise) -> TrainerResult<()> {
        match exercise.id {
            None => self.process_save(exercise).await,
            Some(id) => {
                //Verify that the exercise actually exists.  We don't worry about a transactional
                //context for the query and update for now.  We'll see about adding support in a
                //future iteration
                match self.repo.query_by_id(id).await {
                    Ok(_) => match self.repo.update(exercise).await {
                        Ok(_) => {
                            debug!("update to exercise was successful");
                            Ok(())
                        },
                        Err(err) => match err {
                            RepositoryError::PersistenceError(e) => {
                                error!("{}", e.to_string());
                                Err(TrainerError::PersistenceError(
                                    "an error occurred while updating exercise".to_string(),
                                ))
                            }
                            e => {
                                error!("{}", e.to_string());
                                Err(TrainerError::UnknownError(
                                    "an unknown error occurred while updating exercise".to_string(),
                                ))
                            }
                        },
                    },
                    Err(err) => match err {
                        RepositoryError::ItemNotFoundError => {
                            let err_msg = "exercise was not found with provided id";
                            error!("{}", err_msg);
                            Err(TrainerError::PersistenceError(err_msg.to_string()))
                        },
                        e => {
                            error!("{}", e.to_string());
                            Err(TrainerError::UnknownError(
                                "an error occurred while searching for existing exercise"
                                    .to_string(),
                            ))
                        }
                    },
                }
            }
        }
    }

    // Retrieves an exercise by name (case-insensitive).  Every exercise name *MUST* be unique
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
                    },
                    RepositoryError::ItemNotFoundError => {
                        debug!("exercise not found");
                        Err(ExerciseNotFound(name.clone()))
                    },
                    err => {
                        error!("{}", err.to_string());
                        Err(LookupError("unknown error with repository".to_string()))
                    }
                }
            }
        }
    }

    ///Retrieves a list of exercises
    ///
    ///# Returns
    ///* [`Ok`]` with the list of exercises
    ///* A [`TrainerError::QueryError`] if there is a problem retrieving the list
    #[instrument(skip(self))]
    async fn list(&self) -> TrainerResult<Vec<Exercise>> {
        match self.repo.list().await {
            Ok(exercises) => Ok(exercises),
            Err(err) => {
                error!("{}", err.to_string());
                Err(TrainerError::QueryError(
                    "an error occurred while retrieving the list of exercises".to_string(),
                ))
            }
        }
    }

    ///Deletes the exercise from the repository
    /// # Arguments
    /// * `name`: The name of the exercise to delete
    /// # Returns
    /// * [`Ok`] if the deletion was successful
    /// * [`TrainerError::DeleteError`] if there was a problem deleting the exercise
    /// * [`TrainerError::ExerciseNotFound`] if the exercise was not found
    /// * [`TrainerError::QueryError`] if there was error while looking up the id
    #[instrument(skip(self), fields(name = name))]
    async fn delete(&self, name: String) -> TrainerResult<()> {
        //Get the id by searching the name
        match self.repo.query_by_name(name).await {
            Ok(exercise) => {
                // We can unwrap here because Option MUST BE Some
                match self.repo.delete(exercise.id.unwrap()).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        error!("{}", err.to_string());
                        Err(TrainerError::DeleteError(
                            "an error occurred while deleting the exercise".to_string(),
                        ))
                    }
                }
            }
            Err(err) => match err {
                RepositoryError::ItemNotFoundError => {
                    let err_msg = "exercise was not found";
                    error!("{}", err_msg);
                    Err(TrainerError::ExerciseNotFound(err_msg.to_string()))
                }
                err => {
                    error!("{}", err.to_string());
                    Err(TrainerError::QueryError(
                        "an error occurred while retrieving id for exercise".to_string(),
                    ))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RepositoryError::ItemNotFoundError;
    use crate::{RepositoryError, TrainerError};
    use mockall::Sequence;
    use test_log::test;

    fn deadlift(id: Option<i64>) -> Exercise {
        Exercise {
            id: id,
            name: "Deadlift".to_string(),
            description: Some("A lift made from a standing position, without the use of a bench or other equipment.".to_string()),
            exercise_type: ExerciseType::Barbell,
        }
    }

    fn benchpress(id: Option<i64>) -> Exercise {
        Exercise{
            id: id,
            name: "Benchpress".to_string(),
            description: Some("A lift or exercise in which a weight is raised by extending the arms upward while lying on a bench.".to_string()),
            exercise_type: ExerciseType::Barbell,
        }
    }

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

    #[test(tokio::test)]
    async fn test_save_new_ok() {
        let mut repo = MockExerciseRepository::new();
        repo.expect_create().returning(|_result| Ok(1));
        let mgr = ExerciseManager::new(&repo).unwrap();

        let mut exercise = deadlift(None);
        let result = mgr.save(&mut exercise).await;
        assert!(result.is_ok());
        assert!(matches!(
            exercise.id,
            Some(id) if id == 1
        ));
    }

    #[test(tokio::test)]
    async fn test_save_new_failed() {
        let mut repo = MockExerciseRepository::new();
        repo.expect_create()
            .returning(|_result| Err(RepositoryError::PersistenceError("db error".to_string())));
        let mgr = ExerciseManager::new(&repo).unwrap();

        let mut exercise = deadlift(None);
        let result = mgr.save(&mut exercise).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::PersistenceError(s) if s == "an error occurred while creating a new exercise"
        ));
    }

    #[test(tokio::test)]
    async fn test_save_new_failed_unknown() {
        let mut repo = MockExerciseRepository::new();
        repo.expect_create()
            .returning(|_result| Err(RepositoryError::UnknownError("db error".to_string())));
        let mgr = ExerciseManager::new(&repo).unwrap();

        let mut exercise = deadlift(None);
        let result = mgr.save(&mut exercise).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::UnknownError(s) if s == "an unknown error occurred while creating a new exercise"
        ));
    }

    #[test(tokio::test)]
    async fn test_save_existing_ok() {
        let mut repo = MockExerciseRepository::new();
        let mut seq = Sequence::new();
        let mut dl = deadlift(Some(1000));

        repo.expect_query_by_id()
            .with(eq(1000))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_string| {
                let returned_dl = deadlift(Some(1000));
                Ok(returned_dl)
            });

        repo.expect_update()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_x| Ok(()));
        let mgr = ExerciseManager::new(&repo).unwrap();
        let result = mgr.save(&mut dl).await;
        assert!(result.is_ok());
    }

    #[test(tokio::test)]
    async fn test_save_existing_bad_id() {
        let mut repo = MockExerciseRepository::new();
        let mut seq = Sequence::new();
        let mut dl = deadlift(Some(1000));

        repo.expect_query_by_id()
            .with(eq(1000))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_string| Err(ItemNotFoundError));

        let mgr = ExerciseManager::new(&repo).unwrap();
        let result = mgr.save(&mut dl).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::PersistenceError(s) if s == "exercise was not found with provided id"
        ))
    }

    #[test(tokio::test)]
    async fn test_save_existing_unknown_err() {
        let mut repo = MockExerciseRepository::new();
        let mut seq = Sequence::new();
        let mut dl = deadlift(Some(1000));

        repo.expect_query_by_id()
            .with(eq(1000))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_string| Err(RepositoryError::UnknownError("db error".to_string())));

        let mgr = ExerciseManager::new(&repo).unwrap();
        let result = mgr.save(&mut dl).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::UnknownError(s) if s == "an error occurred while searching for existing exercise"
        ))
    }

    #[test(tokio::test)]
    async fn test_save_existing_failed_update() {
        let mut repo = MockExerciseRepository::new();
        let mut seq = Sequence::new();
        let mut dl = deadlift(Some(1000));

        repo.expect_query_by_id()
            .with(eq(1000))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_string| {
                let returned_dl = deadlift(Some(1000));
                Ok(returned_dl)
            });

        repo.expect_update()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_x| Err(RepositoryError::PersistenceError("db error".to_string())));
        let mgr = ExerciseManager::new(&repo).unwrap();
        let result = mgr.save(&mut dl).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::PersistenceError(s) if s == "an error occurred while updating exercise"
        ))
    }

    #[test(tokio::test)]
    async fn test_save_existing_unknown_update_failure() {
        let mut repo = MockExerciseRepository::new();
        let mut seq = Sequence::new();
        let mut dl = deadlift(Some(1000));

        repo.expect_query_by_id()
            .with(eq(1000))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_string| {
                let returned_dl = deadlift(Some(1000));
                Ok(returned_dl)
            });

        repo.expect_update()
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_x| Err(RepositoryError::UnknownError("db error".to_string())));
        let mgr = ExerciseManager::new(&repo).unwrap();
        let result = mgr.save(&mut dl).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::UnknownError(s) if s == "an unknown error occurred while updating exercise"
        ))
    }

    #[test(tokio::test)]
    async fn list_ok() {
        let mut repo = MockExerciseRepository::new();

        repo.expect_list().returning(|| {
            let dl = deadlift(Some(1000));
            let bp = benchpress(Some(2000));
            Ok(vec![dl, bp])
        });

        let mgr = ExerciseManager::new(&repo).unwrap();
        let result = mgr.list().await;
        assert!(result.is_ok());
        let exercises = result.unwrap();
        assert_eq!(2, exercises.len());
        assert!(exercises.contains(&deadlift(Some(1000))));
        assert!(exercises.contains(&benchpress(Some(2000))));
    }

    #[test(tokio::test)]
    async fn list_failed() {
        let mut repo = MockExerciseRepository::new();

        repo.expect_list()
            .returning(|| Err(RepositoryError::UnknownError("db error".to_string())));
        let mgr = ExerciseManager::new(&repo).unwrap();
        let result = mgr.list().await;

        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::QueryError(s) if s == "an error occurred while retrieving the list of exercises"
        ))
    }

    #[test(tokio::test)]
    async fn delete_ok() {
        let mut repo = MockExerciseRepository::new();
        let mut seq = Sequence::new();

        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_string| Ok(deadlift(Some(1000))));

        repo.expect_delete()
            .with(eq(1000))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Ok(()));

        let mgr = ExerciseManager::new(&repo).unwrap();
        let dl = deadlift(Some(1000));
        let result = mgr.delete(dl.name).await;
        assert!(result.is_ok());
    }

    #[test(tokio::test)]
    async fn delete_failed() {
        let mut repo = MockExerciseRepository::new();
        let mut seq = Sequence::new();

        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_string| Ok(deadlift(Some(1000))));

        repo.expect_delete()
            .with(eq(1000))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_| Err(RepositoryError::DeleteError("db error".to_string())));

        let mgr = ExerciseManager::new(&repo).unwrap();
        let dl = deadlift(Some(1000));
        let result = mgr.delete(dl.name).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::DeleteError(s) if s == "an error occurred while deleting the exercise"
        ))
    }

    #[test(tokio::test)]
    async fn delete_failed_item_not_found() {
        let mut repo = MockExerciseRepository::new();
        let mut seq = Sequence::new();

        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .times(1)
            .in_sequence(&mut seq)
            .returning(|_string| Err(RepositoryError::ItemNotFoundError));

        repo.expect_delete()
            .with(eq(1000))
            .times(0)
            .in_sequence(&mut seq)
            .returning(|_| Err(RepositoryError::DeleteError("db error".to_string())));

        let mgr = ExerciseManager::new(&repo).unwrap();
        let dl = deadlift(Some(1000));
        let result = mgr.delete(dl.name).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::ExerciseNotFound(_)
        ))
    }

    #[test(tokio::test)]
    async fn delete_failed_query_failure() {
        let mut repo = MockExerciseRepository::new();

        repo.expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .times(1)
            .returning(|_string| Err(RepositoryError::UnknownError("db error".to_string())));

        let mgr = ExerciseManager::new(&repo).unwrap();
        let dl = deadlift(Some(1000));
        let result = mgr.delete(dl.name).await;
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            TrainerError::QueryError(s) if s == "an error occurred while retrieving id for exercise"
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
