#[cfg(test)]
mod exercise_tests {
    use api::exercise::ExerciseType::Barbell;
    use api::exercise::{Exercise, ExerciseManagement, ExerciseManager};
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use sqlite::{DBType, SqliteExerciseRepository};
    use tempfile::tempdir;
    use test_log::test;

    fn db_name() -> String {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        format!("testdb-{}.db3", rand_string)
    }

    fn deadlift(id: Option<i64>) -> Exercise {
        Exercise {
            id,
            name: "Deadlift".to_string(),
            description: None,
            exercise_type: Barbell,
        }
    }

    fn benchpress(id: Option<i64>) -> Exercise {
        Exercise {
            id,
            name: "Benchpress".to_string(),
            description: None,
            exercise_type: Barbell,
        }
    }

    fn squat(id: Option<i64>) -> Exercise {
        Exercise {
            id,
            name: "Squat".to_string(),
            description: None,
            exercise_type: Barbell,
        }
    }

    #[test(tokio::test)]
    async fn create_exercise_ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo_result = SqliteExerciseRepository::new(DBType::File(file_path.as_path())).await;
        let repo = repo_result.unwrap();
        let mgr = ExerciseManager::new(&repo).unwrap();

        let mut dl = deadlift(None);
        let create_result = mgr.save(&mut dl).await;
        assert!(create_result.is_ok());
        assert!(matches!(dl.id, Some(_)));
    }

    #[test(tokio::test)]
    async fn create_many_exercises_ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo_result = SqliteExerciseRepository::new(DBType::File(file_path.as_path())).await;
        let repo = repo_result.unwrap();
        let mgr = ExerciseManager::new(&repo).unwrap();

        let mut dl = deadlift(None);
        let mut bp = benchpress(None);
        let mut sq = squat(None);
        let exercises = vec![&mut dl, &mut bp, &mut sq];

        for exercise in exercises {
            let create_result = mgr.save(exercise).await;
            assert!(create_result.is_ok());
            assert!(matches!(exercise.id, Some(_)));
        }
    }
}
