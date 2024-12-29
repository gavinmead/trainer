use api::{Exercise, ExerciseType, Repository, TrainerError::*, TrainerResult};
use rusqlite::{params, Connection, Error, Row};
use std::path::Path;

const CREATE_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS EXERCISE (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    exercise_type INTEGER NOT NULL
)
";

const INSERT: &str = "\
INSERT INTO EXERCISE (name, description, exercise_type) VALUES (?1, ?2, ?3)
";

const SELECT_BY_NAME: &str = "\
SELECT id, name, description, exercise_type
FROM EXERCISE WHERE name = :name COLLATE NOCASE
";

const SELECT_BY_ID: &str = "\
SELECT id, name, description, exercise_type
FROM EXERCISE WHERE id = :id
";

const SELECT_ALL: &str = "\
    SELECT id, name, description, exercise_type
    FROM EXERCISE
";

pub enum DBType<'a> {
    InMemory,
    File(&'a Path),
}

pub struct SqliteExerciseRepository {
    conn: Connection,
}

impl SqliteExerciseRepository {
    fn process_connection(conn_result: Result<Connection, Error>) -> TrainerResult<Connection> {
        match conn_result {
            Ok(c) => Ok(c),
            Err(e) => Err(ConnectionError(e.to_string())),
        }
    }

    pub fn new(db_type: DBType) -> TrainerResult<Self> {
        let conn: TrainerResult<Connection> = match db_type {
            DBType::InMemory => Self::process_connection(Connection::open_in_memory()),
            DBType::File(p) => Self::process_connection(Connection::open(p)),
        };

        match conn {
            Ok(c) => {
                //Create the table
                match c.execute(CREATE_TABLE, ()) {
                    Ok(_) => Ok(SqliteExerciseRepository { conn: c }),
                    Err(e) => Err(PersistenceError(e.to_string())),
                }
            }
            Err(e) => Err(e),
        }
    }

    fn map_exercise(row: &Row) -> Result<Option<Exercise>, Error> {
        let id: i64 = row.get(0).unwrap();
        let name = row.get(1).unwrap();
        let description: String = row.get(2).unwrap();
        let et: i64 = row.get(3).unwrap();

        let mut final_description = None;
        if !description.is_empty() {
            final_description = Some(description);
        }

        let exercise_type: ExerciseType = et.into();

        Ok(Some(Exercise {
            id: Some(id),
            name,
            description: final_description,
            exercise_type,
        }))
    }

    fn handle_row(result: Result<Option<Exercise>, Error>) -> TrainerResult<Option<Exercise>> {
        match result {
            Ok(r) => Ok(r),
            Err(Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(QueryError(e.to_string())),
        }
    }
}

impl Repository<Exercise> for SqliteExerciseRepository {
    fn create(&self, exercise: &Exercise) -> TrainerResult<i64> {
        let description = match &exercise.description {
            None => "",
            Some(x) => x,
        };
        let e_type: i64 = exercise.exercise_type.clone().into();
        match self
            .conn
            .execute(INSERT, params![exercise.name, description, e_type])
        {
            Ok(_) => {
                let id = self.conn.last_insert_rowid();
                Ok(id)
            }
            Err(e) => Err(PersistenceError(e.to_string())),
        }
    }

    fn query_by_name(&self, name: String) -> TrainerResult<Option<Exercise>> {
        let mut stmt = self.conn.prepare(SELECT_BY_NAME).unwrap();
        let row = stmt.query_row(&[(":name", &name)], Self::map_exercise);
        Self::handle_row(row)
    }

    fn query_by_id(&self, id: i64) -> TrainerResult<Option<Exercise>> {
        let mut stmt = self.conn.prepare(SELECT_BY_ID).unwrap();
        let row = stmt.query_row(&[(":id", &id)], Self::map_exercise);
        Self::handle_row(row)
    }

    fn list(&self) -> TrainerResult<Vec<Exercise>> {
        let mut stmt = self.conn.prepare(SELECT_ALL).unwrap();
        let row_result = stmt.query_map([], Self::map_exercise);
        match row_result {
            Ok(rows) => {
                let mut v: Vec<Exercise> = Vec::new();
                for row in rows {
                    v.push(row.unwrap().unwrap())
                }
                Ok(v)
            }
            Err(e) => Err(QueryError(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DBType::{File, InMemory};
    use api::ExerciseType::Barbell;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use rstest::*;
    use std::path::PathBuf;
    use tempfile::{tempdir, TempDir};

    struct TestConfig {
        dir: TempDir,
        file_path: PathBuf,
        repo: SqliteExerciseRepository,
    }

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
            exercise_type: ExerciseType::Barbell,
        }
    }

    #[fixture]
    fn kbswing() -> Exercise {
        Exercise {
            id: None,
            name: "Two-Arm Kettlebell Swing".to_string(),
            description: None,
            exercise_type: ExerciseType::KettleBell,
        }
    }

    #[fixture]
    fn test_config() -> TestConfig {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(File(file_path.as_path())).unwrap();
        TestConfig {
            dir: dir,
            file_path: file_path,
            repo: repo,
        }
    }

    #[test]
    fn new_in_memory_connection() {
        let repo_result = SqliteExerciseRepository::new(InMemory);
        assert!(repo_result.is_ok())
    }

    #[rstest]
    fn new_with_file(db_name: String) {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name);
        let repo_result = SqliteExerciseRepository::new(File(file_path.as_path()));
        assert!(repo_result.is_ok());
    }

    #[rstest]
    fn new_failure(db_name: String) {
        let dir = tempdir().unwrap();
        let bad_path = dir.path().join("doesnotexist");
        let file_path = bad_path.join(db_name);
        let repo_result = SqliteExerciseRepository::new(File(file_path.clone().as_path()));
        assert!(repo_result.is_err());

        let expected_error = format!(
            "unable to open database file: {}",
            file_path.into_os_string().into_string().unwrap()
        );
        assert!(matches!(
            repo_result.err().unwrap(),
            ConnectionError(s) if s == expected_error
        ));
    }

    #[rstest]
    fn creation_ok(test_config: TestConfig) {
        let repo = test_config.repo;
        let result = repo.create(&Exercise {
            id: None,
            name: "Deadlift".to_string(),
            description: None,
            exercise_type: Barbell,
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[rstest]
    fn query_by_name_ok(test_config: TestConfig, deadlift: Exercise) {
        let repo = test_config.repo;
        repo.create(&deadlift).unwrap();

        let queries = vec!["Deadlift", "deadlift", "DeadLift", "DEADLIFT", "dEaDlIfT"];

        for q in queries {
            let query_result = repo.query_by_name(q.to_string());
            assert!(query_result.is_ok());

            let exercise = query_result.unwrap().unwrap();
            assert_eq!(exercise.id, Some(1));
            assert_eq!(exercise.name, "Deadlift");
            assert_eq!(exercise.description, None);
            assert_eq!(exercise.exercise_type, Barbell);
        }
    }

    #[rstest]
    fn query_a_kb_ok(test_config: TestConfig, kbswing: Exercise) {
        let repo = test_config.repo;
        let result = repo.create(&kbswing).unwrap();
        let query_result = repo.query_by_id(result);
        assert!(query_result.is_ok());

        let exercise = query_result.unwrap().unwrap();
        assert_eq!(exercise.id, Some(1));
        assert_eq!(exercise.name, kbswing.name);
        assert_eq!(exercise.description, None);
        assert_eq!(exercise.exercise_type, kbswing.exercise_type);
    }

    #[rstest]
    fn query_by_name_not_found(test_config: TestConfig) {
        let result = test_config.repo.query_by_name("not_found".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[rstest]
    fn query_by_name_with_description(test_config: TestConfig) {
        let repo = test_config.repo;
        repo.create(&Exercise{
            id: None,
            name: "Deadlift".to_string(),
            description: Some("a lift made from a standing position, without the use of a bench or other equipment.".to_string()),
            exercise_type: Barbell,
        }).unwrap();

        let query_result = repo.query_by_name("Deadlift".to_string());
        assert!(query_result.is_ok());

        let exercise = query_result.unwrap().unwrap();
        assert_eq!(exercise.id, Some(1));
        assert_eq!(exercise.name, "Deadlift");
        assert_eq!(exercise.description, Some("a lift made from a standing position, without the use of a bench or other equipment.".to_string()));
        assert_eq!(exercise.exercise_type, Barbell);
    }

    #[rstest]
    fn query_by_id_ok(test_config: TestConfig, deadlift: Exercise) {
        let repo = test_config.repo;
        let result = repo.create(&deadlift).unwrap();

        let query_result = repo.query_by_id(result);
        assert!(query_result.is_ok());

        let exercise = query_result.unwrap().unwrap();
        assert_eq!(exercise.id, Some(1));
        assert_eq!(exercise.name, "Deadlift");
        assert_eq!(exercise.description, None);
        assert_eq!(exercise.exercise_type, Barbell);
    }

    #[rstest]
    fn query_by_id_not_found(test_config: TestConfig) {
        let repo = test_config.repo;
        let query_result = repo.query_by_id(2000);
        assert!(query_result.is_ok());
        assert_eq!(query_result.unwrap(), None);
    }

    #[rstest]
    fn list_ok(test_config: TestConfig, deadlift: Exercise, benchpress: Exercise) {
        let repo = test_config.repo;
        let _ = repo.create(&deadlift);
        let _ = repo.create(&benchpress);

        let list_result = repo.list();
        assert!(list_result.is_ok());
        let exercises = list_result.unwrap();
        assert_eq!(2, exercises.len());

        for ex in exercises {
            assert!(matches!(
                    ex.id,
                    Some(i) if i > 0
            ));
        }
    }
}
