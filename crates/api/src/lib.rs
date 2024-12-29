#[cfg(test)]
use mockall::{automock, predicate::*};

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] //this is temporary until code base evolves
pub struct Exercise {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub exercise_type: ExerciseType,
}

pub type TrainerResult<T> = Result<T, TrainerError>;

#[derive(thiserror::Error, Debug, Clone)]
pub enum TrainerError {
    #[error("ExerciseNotFound: {0}")]
    ExerciseNotFound(String),

    #[error("PersistenceError: {0}")]
    PersistenceError(String),

    #[error("ConnectionError: {0}")]
    ConnectionError(String),

    #[error("QueryError: {0}")]
    QueryError(String),
}

pub trait ExerciseManagement {
    fn create(&self, exercise: &mut Exercise) -> TrainerResult<()>;

    fn get_by_name(&self, name: String) -> TrainerResult<Exercise>;

    fn get_by_id(&self, id: i64) -> TrainerResult<Exercise>;

    fn list(&self) -> TrainerResult<Vec<Exercise>>;
}

pub struct ExerciseManager {
    repository: Box<ExerciseRepository>,
}

impl ExerciseManager {
    pub fn new(exercise_repository: Box<ExerciseRepository>) -> TrainerResult<ExerciseManager> {
        Ok(ExerciseManager {
            repository: exercise_repository,
        })
    }

    fn process_query_result(
        result: TrainerResult<Option<Exercise>>,
        error_message: String,
    ) -> TrainerResult<Exercise> {
        match result {
            Ok(r) => match r {
                None => Err(TrainerError::ExerciseNotFound(error_message)),
                Some(e) => Ok(e),
            },
            Err(e) => Err(e),
        }
    }
}

impl ExerciseManagement for ExerciseManager {
    fn create(&self, exercise: &mut Exercise) -> TrainerResult<()> {
        let result = self.repository.create(exercise);
        match result {
            Ok(r) => {
                exercise.id = Some(r);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn get_by_name(&self, name: String) -> TrainerResult<Exercise> {
        let result = self.repository.query_by_name(name.clone());
        Self::process_query_result(result, name)
    }

    fn get_by_id(&self, id: i64) -> TrainerResult<Exercise> {
        let result = self.repository.query_by_id(id);
        Self::process_query_result(result, format!("exercise with id {} was not found", id))
    }

    fn list(&self) -> TrainerResult<Vec<Exercise>> {
        self.repository.list()
    }
}

#[cfg_attr(test, automock)]
pub trait Repository<T> {
    /// Persists T.
    /// Will return the repository generated ID in a TrainerResult.
    /// TrainerError will be a PersistenceError
    fn create(&self, exercise: &T) -> TrainerResult<i64>;

    fn query_by_name(&self, name: String) -> TrainerResult<Option<T>>;

    fn query_by_id(&self, id: i64) -> TrainerResult<Option<T>>;

    fn list(&self) -> TrainerResult<Vec<T>>;
}

pub type ExerciseRepository = dyn Repository<Exercise>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ExerciseType::Barbell;
    use crate::TrainerError::{ExerciseNotFound, PersistenceError};

    #[test]
    fn create_exercise_manager() {
        let mock_repo = MockRepository::<Exercise>::new();
        let mgr_result = ExerciseManager::new(Box::new(mock_repo));
        assert!(mgr_result.is_ok());
    }

    #[test]
    fn create_exercise() {
        let mut exercise = Exercise {
            id: None,
            name: "Deadlift".to_string(),
            description: None,
            exercise_type: Barbell,
        };

        let mut mock_repo = MockRepository::<Exercise>::new();
        mock_repo.expect_create().returning(|_x| Ok(1));
        let mgr_result = ExerciseManager::new(Box::new(mock_repo));

        let mgr = mgr_result.unwrap();
        let result = mgr.create(&mut exercise);
        assert!(result.is_ok());
        assert_eq!(exercise.id.unwrap(), 1);
        assert_eq!(exercise.name, "Deadlift".to_string());
        assert!(exercise.description.is_none());
        assert_eq!(exercise.exercise_type, Barbell);
    }

    #[test]
    fn create_exercise_failed() {
        let mut exercise = Exercise {
            id: None,
            name: "Deadlift".to_string(),
            description: None,
            exercise_type: Barbell,
        };

        let mut mock_repo = MockRepository::<Exercise>::new();
        mock_repo
            .expect_create()
            .returning(|_x| Err(PersistenceError("unable to create exercise".to_string())));
        let mgr_result = ExerciseManager::new(Box::new(mock_repo));

        let mgr = mgr_result.unwrap();
        let result = mgr.create(&mut exercise);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            PersistenceError(s) if s == "unable to create exercise"
        ));
    }

    #[test]
    fn get_exercise_by_name_ok() {
        let mut mock_repo = MockRepository::<Exercise>::new();
        mock_repo
            .expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .returning(|_string| {
                Ok(Some(Exercise {
                    id: Some(1),
                    name: "Deadlift".to_string(),
                    description: None,
                    exercise_type: Barbell,
                }))
            });

        let mgr = ExerciseManager::new(Box::new(mock_repo)).unwrap();

        let exercise_result = mgr.get_by_name("Deadlift".to_string());
        assert!(exercise_result.is_ok());
        let found_exercise = exercise_result.unwrap();
        assert_eq!(found_exercise.id, Some(1));
        assert_eq!(found_exercise.name, "Deadlift".to_string());
        assert_eq!(found_exercise.description, None);
        assert_eq!(found_exercise.exercise_type, Barbell);
    }

    #[test]
    fn get_exercise_by_name_not_found() {
        let mut mock_repo = MockRepository::<Exercise>::new();
        mock_repo
            .expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .returning(|_string| Ok(None));

        let mgr = ExerciseManager::new(Box::new(mock_repo)).unwrap();

        let exercise_result = mgr.get_by_name("Deadlift".to_string());
        assert!(exercise_result.is_err());
        assert!(matches!(
            exercise_result.err().unwrap(),
            ExerciseNotFound(s) if s == "Deadlift"
        ));
    }

    #[test]
    fn get_exercise_by_name_query_error() {
        let mut mock_repo = MockRepository::<Exercise>::new();
        mock_repo
            .expect_query_by_name()
            .with(eq("Deadlift".to_string()))
            .returning(|_string| Err(PersistenceError("error".to_string())));

        let mgr = ExerciseManager::new(Box::new(mock_repo)).unwrap();

        let exercise_result = mgr.get_by_name("Deadlift".to_string());
        assert!(exercise_result.is_err());
        assert!(matches!(
            exercise_result.err().unwrap(),
            PersistenceError(s) if s == "error"
        ));
    }

    #[test]
    fn get_exercise_by_id_ok() {
        let mut mock_repo = MockRepository::<Exercise>::new();
        mock_repo
            .expect_query_by_id()
            .with(eq(1))
            .returning(|_string| {
                Ok(Some(Exercise {
                    id: Some(1),
                    name: "Deadlift".to_string(),
                    description: None,
                    exercise_type: Barbell,
                }))
            });
        let mgr = ExerciseManager::new(Box::new(mock_repo)).unwrap();

        let exercise_result = mgr.get_by_id(1);
        assert!(exercise_result.is_ok());
        let found_exercise = exercise_result.unwrap();
        assert_eq!(found_exercise.id, Some(1));
        assert_eq!(found_exercise.name, "Deadlift".to_string());
        assert_eq!(found_exercise.description, None);
        assert_eq!(found_exercise.exercise_type, Barbell);
    }

    #[test]
    fn get_exercise_by_id_not_found() {
        let mut mock_repo = MockRepository::<Exercise>::new();
        mock_repo
            .expect_query_by_id()
            .with(eq(1))
            .returning(|_string| Ok(None));

        let mgr = ExerciseManager::new(Box::new(mock_repo)).unwrap();

        let exercise_result = mgr.get_by_id(1);
        assert!(exercise_result.is_err());
        assert!(matches!(
            exercise_result.err().unwrap(),
            ExerciseNotFound(s) if s == "exercise with id 1 was not found"
        ));
    }

    #[test]
    fn get_exercise_by_id_query_error() {
        let mut mock_repo = MockRepository::<Exercise>::new();
        mock_repo
            .expect_query_by_id()
            .with(eq(1))
            .returning(|_string| Err(PersistenceError("error".to_string())));

        let mgr = ExerciseManager::new(Box::new(mock_repo)).unwrap();

        let exercise_result = mgr.get_by_id(1);
        assert!(exercise_result.is_err());
        assert!(matches!(
            exercise_result.err().unwrap(),
            PersistenceError(s) if s == "error"
        ));
    }

    #[test]
    fn list_exercises_ok() {
        let mut mock_repo = MockRepository::<Exercise>::new();
        mock_repo.expect_list().returning(|| {
            let result = vec![
                Exercise {
                    id: Some(1),
                    name: "Deadlift".to_string(),
                    description: None,
                    exercise_type: ExerciseType::Barbell,
                },
                Exercise {
                    id: Some(2),
                    name: "BenchPress".to_string(),
                    description: None,
                    exercise_type: ExerciseType::Barbell,
                },
            ];

            Ok(result)
        });
        let mgr = ExerciseManager::new(Box::new(mock_repo)).unwrap();

        let exercises_result = mgr.list();
        assert!(exercises_result.is_ok());

        let exercises = exercises_result.unwrap();
        assert_eq!(2, exercises.len());
    }

    #[test]
    #[should_panic]
    fn test_bad_i64_for_exercise_type() {
        ExerciseType::from(1000);
    }
}
