#[cfg(test)]
use mockall::{automock, predicate::*};

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub enum ExerciseType {
    Barbell,
    KettleBell,
}

#[derive(Clone, Debug)]
pub struct Exercise {
    id: Option<i64>,
    name: String,
    description: Option<String>,
    exercise_type: ExerciseType,
}

pub type TrainerResult<T> = Result<T, TrainerError>;

#[derive(thiserror::Error, Debug, Clone)]
pub enum TrainerError {
    #[error("ExerciseNotFound: {0}")]
    ExerciseNotFound(String),
}

pub trait ExerciseManagement {
    fn create(&mut self, exercise: &mut Exercise) -> TrainerResult<()>;

    fn get_by_name(&self, name: String) -> TrainerResult<Exercise>;

    fn get_by_id(&self, id: i64) -> TrainerResult<Exercise>;

    fn list(&self) -> TrainerResult<Vec<Exercise>>;
}

pub struct ExerciseManager {
    repository: Box<ExerciseRepository>
}

impl ExerciseManager {
    pub fn new(exercise_repository: Box<ExerciseRepository>) -> TrainerResult<ExerciseManager> {
        Ok(ExerciseManager{
            repository: exercise_repository,
        })
    }
}

impl ExerciseManagement for ExerciseManager {
    fn create(&mut self, exercise: &mut Exercise) -> TrainerResult<()> {
        let result = self.repository.create(exercise);
        exercise.id = Some(result);
        Ok(())
    }

    fn get_by_name(&self, name: String) -> TrainerResult<Exercise> {
        todo!()
    }

    fn get_by_id(&self, id: i64) -> TrainerResult<Exercise> {
        todo!()
    }

    fn list(&self) -> TrainerResult<Vec<Exercise>> {
        todo!()
    }
}

#[cfg_attr(test, automock)]
pub trait Repository<T> {
    fn create(&mut self, exercise: &T) -> i64;

    fn query_by_name(&self, name: String) -> TrainerResult<T>;

    fn query_by_id(&self, id: i64) -> TrainerResult<T>;

    fn list(&self) -> TrainerResult<T>;
}

pub type ExerciseRepository = dyn Repository<Exercise>;

#[cfg(test)]
mod tests {
    use crate::ExerciseType::Barbell;
    use super::*;

    #[test]
    fn create_exercise_manager() {
        let mock_repo = MockRepository::<Exercise>::new();
        let mgr_result = ExerciseManager::new(Box::new(mock_repo));
        assert!(mgr_result.is_ok());
    }

    #[test]
    fn create_exercise() {
        let mut exercise = Exercise{
            id: None,
            name: "Deadlift".to_string(),
            description: None,
            exercise_type: Barbell,
        };

        let mut mock_repo = MockRepository::<Exercise>::new();
        mock_repo.expect_create().returning(|_x| {
            1
        });
        let mgr_result = ExerciseManager::new(Box::new(mock_repo));

        let mut mgr = mgr_result.unwrap();
        let result = mgr.create(&mut exercise);
        assert!(result.is_ok());
        assert_eq!(exercise.id.unwrap(), 1);
        assert_eq!(exercise.name, "Deadlift".to_string());
        assert!(exercise.description.is_none());
        assert_eq!(exercise.exercise_type, Barbell);
    }
}
