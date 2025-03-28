use crate::exercise::error;
use crate::repository::ExerciseRepository;
use crate::{Exercise, ExerciseError, RepositoryError};
use async_trait::async_trait;
use error::ExerciseResult;
use tracing::{debug, error, instrument};

#[async_trait]
pub trait ExerciseManagement {
    // Will create or update an exercise
    async fn save(&self, exercise: &mut Exercise) -> ExerciseResult<()>;

    async fn get_by_name(&self, name: String) -> ExerciseResult<Exercise>;

    async fn list(&self) -> ExerciseResult<Vec<Exercise>>;

    async fn delete(&self, name: String) -> ExerciseResult<()>;
}

#[derive(Clone, Debug)]
pub struct ExerciseManager<'a, T: ExerciseRepository> {
    repo: &'a T,
}

impl<'a, T: ExerciseRepository> ExerciseManager<'a, T> {
    #[allow(dead_code)]
    pub fn new(repo: &'a T) -> ExerciseResult<Self> {
        Ok(Self { repo })
    }

    async fn process_save(&self, exercise: &mut Exercise) -> ExerciseResult<()> {
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
                    Err(ExerciseError::SaveFailed)
                }
                e => {
                    error!("{}", e.to_string());
                    Err(ExerciseError::UnknownError)
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
    async fn save(&self, exercise: &mut Exercise) -> ExerciseResult<()> {
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
                        }
                        Err(err) => match err {
                            RepositoryError::PersistenceError(e) => {
                                error!("{}", e.to_string());
                                Err(ExerciseError::SaveFailed)
                            }
                            e => {
                                error!("{}", e.to_string());
                                Err(ExerciseError::UnknownError)
                            }
                        },
                    },
                    Err(err) => match err {
                        RepositoryError::ItemNotFoundError => {
                            let err_msg = "exercise was not found with provided id";
                            error!("{}", err_msg);
                            Err(ExerciseError::ExerciseNotFoundError)
                        }
                        e => {
                            error!("{}", e.to_string());
                            Err(ExerciseError::UnknownError)
                        }
                    },
                }
            }
        }
    }

    // Retrieves an exercise by name (case-insensitive).  Every exercise name *MUST* be unique
    #[instrument(skip(self), fields(name = name))]
    async fn get_by_name(&self, name: String) -> ExerciseResult<Exercise> {
        match self.repo.query_by_name(name.clone()).await {
            Ok(e) => {
                debug!("exercise found");
                Ok(e)
            }
            Err(err) => {
                match err {
                    RepositoryError::ConnectionError(e) => {
                        //log the backend error message
                        error!("{}", e);
                        Err(ExerciseError::LookupError)
                    }
                    RepositoryError::ItemNotFoundError => {
                        debug!("exercise not found");
                        Err(ExerciseError::ExerciseNotFoundError)
                    }
                    err => {
                        error!("{}", err.to_string());
                        Err(ExerciseError::LookupError)
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
    async fn list(&self) -> ExerciseResult<Vec<Exercise>> {
        match self.repo.list().await {
            Ok(exercises) => Ok(exercises),
            Err(err) => {
                error!("{}", err.to_string());
                Err(ExerciseError::LookupError)
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
    async fn delete(&self, name: String) -> ExerciseResult<()> {
        //Get the id by searching the name
        match self.repo.query_by_name(name).await {
            Ok(exercise) => {
                // We can unwrap here because Option MUST BE Some
                match self.repo.delete(exercise.id.unwrap()).await {
                    Ok(_) => Ok(()),
                    Err(err) => {
                        error!("{}", err.to_string());
                        Err(ExerciseError::DeleteFailed)
                    }
                }
            }
            Err(err) => match err {
                RepositoryError::ItemNotFoundError => {
                    let err_msg = "exercise was not found";
                    error!("{}", err_msg);
                    Err(ExerciseError::ExerciseNotFoundError)
                }
                err => {
                    error!("{}", err.to_string());
                    Err(ExerciseError::UnknownError)
                }
            },
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExerciseError::ExerciseNotFoundError;
    use crate::RepositoryError::ItemNotFoundError;
    use crate::{ExerciseType, MockExerciseRepository};
    use mockall::predicate::eq;
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
        assert!(matches!(result.err().unwrap(), ExerciseNotFoundError,));
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
        assert!(matches!(result.err().unwrap(), ExerciseError::LookupError))
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
        assert!(matches!(result.err().unwrap(), ExerciseError::LookupError))
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
        assert!(matches!(result.err().unwrap(), ExerciseError::SaveFailed));
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
        assert!(matches!(result.err().unwrap(), ExerciseError::UnknownError));
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
        assert!(matches!(result.err().unwrap(), ExerciseNotFoundError))
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
        assert!(matches!(result.err().unwrap(), ExerciseError::UnknownError))
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
        assert!(matches!(result.err().unwrap(), ExerciseError::SaveFailed))
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
        assert!(matches!(result.err().unwrap(), ExerciseError::UnknownError))
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
        assert!(matches!(result.err().unwrap(), ExerciseError::LookupError))
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
        assert!(matches!(result.err().unwrap(), ExerciseError::DeleteFailed))
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
            ExerciseError::ExerciseNotFoundError
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
        assert!(matches!(result.err().unwrap(), ExerciseError::UnknownError))
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
