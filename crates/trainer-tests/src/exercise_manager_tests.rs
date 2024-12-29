#[cfg(test)]
pub mod tests {
    use api::ExerciseType::{Barbell, KettleBell};
    use api::TrainerError::ExerciseNotFound;
    use api::{Exercise, ExerciseManagement, ExerciseManager};
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use rstest::{fixture, rstest};
    use sqlite::DBType::File;
    use sqlite::SqliteExerciseRepository;
    use tempfile::{tempdir, TempDir};

    #[fixture]
    fn db_name() -> String {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        format!("testdb-{}.db3", rand_string)
    }

    #[fixture]
    fn test_config(db_name: String) -> (ExerciseManager, TempDir) {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name);
        println!(
            "FilePath for DB: {}",
            file_path.clone().into_os_string().into_string().unwrap()
        );
        let repo = SqliteExerciseRepository::new(File(file_path.as_path())).unwrap();
        //We have to pass the dir back to keep it is scope.
        (ExerciseManager::new(Box::new(repo)).unwrap(), dir)
    }

    #[fixture]
    fn deadlift() -> Exercise {
        Exercise {
            id: None,
            name: "Deadlift".to_string(),
            description: None,
            exercise_type: Barbell,
        }
    }

    #[fixture]
    fn benchpress() -> Exercise {
        Exercise {
            id: None,
            name: "Benchpress".to_string(),
            description: None,
            exercise_type: Barbell,
        }
    }

    #[fixture]
    fn kbswing() -> Exercise {
        Exercise {
            id: None,
            name: "Two-Arm Kettlebell Swing".to_string(),
            description: None,
            exercise_type: KettleBell,
        }
    }

    #[fixture]
    fn exercises(kbswing: Exercise, benchpress: Exercise, deadlift: Exercise) -> Vec<Exercise> {
        vec![kbswing, benchpress, deadlift]
    }

    #[rstest]
    fn create_and_get_ok(test_config: (ExerciseManager, TempDir), mut deadlift: Exercise) {
        let mgr = test_config.0;
        let result = mgr.create(&mut deadlift);
        assert!(result.is_ok());
        assert!(matches!(
            deadlift.id,
            Some(i) if i > 0
        ));
    }

    #[rstest]
    fn get_by_name_not_found(test_config: (ExerciseManager, TempDir)) {
        let mgr = test_config.0;
        let result = mgr.get_by_name("Deadlift".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ExerciseNotFound(s) if s == "Deadlift"
        ));
    }

    #[rstest]
    fn get_by_id_not_found(test_config: (ExerciseManager, TempDir)) {
        let mgr = test_config.0;
        let result = mgr.get_by_id(1000);
        assert!(result.is_err());
        assert!(matches!(
            result.err().unwrap(),
            ExerciseNotFound(s) if s == "exercise with id 1000 was not found"
        ));
    }

    #[rstest]
    fn list_ok(test_config: (ExerciseManager, TempDir), exercises: Vec<Exercise>) {
        let mgr = test_config.0;
        for mut e in exercises {
            let result = mgr.create(&mut e);
            assert!(result.is_ok());
            assert!(matches!(
                e.id,
                Some(i) if i > 0
            ));
        }

        let found_exercises = mgr.list();
        assert!(found_exercises.is_ok());
        for e in found_exercises.unwrap() {
            assert!(matches!(
                e.id,
                Some(i) if i > 0
            ));
        }
    }
}
