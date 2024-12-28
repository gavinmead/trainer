use api::{Exercise, ExerciseType, Repository, TrainerError::*, TrainerResult};
use rusqlite::{params, Connection, Error};
use std::path::Path;
use api::ExerciseType::{Barbell, KettleBell};

const CREATE_TABLE: &str = "\
CREATE TABLE IF NOT EXISTS EXERCISE (
    id INTEGER PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    description TEXT,
    is_barbell INTEGER NOT NULL,
    is_kettlebell INTEGER NOT NULL
)
";

const INSERT: &str = "\
INSERT INTO EXERCISE (name, description, is_barbell, is_kettlebell) VALUES (?1, ?2, ?3, ?4)
";

const SELECT_BY_NAME: &str = "\
SELECT id, name, description, is_barbell, is_kettlebell
FROM EXERCISE WHERE name = :name COLLATE NOCASE
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
}

impl Repository<Exercise> for SqliteExerciseRepository {
    fn create(&self, exercise: &Exercise) -> TrainerResult<i64> {
        let mut is_kb = false;
        let mut is_bb = false;

        match exercise.exercise_type {
            ExerciseType::Barbell => is_bb = true,
            ExerciseType::KettleBell => is_kb = true,
        }

        let description = match &exercise.description {
            None => { "" }
            Some(x) => {x}
        };
        match self.conn.execute(
            INSERT,
            params![
                exercise.name,
                description,
                is_bb,
                is_kb,
            ],
        ) {
            Ok(_) => {
                let id = self.conn.last_insert_rowid();
                Ok(id)
            }
            Err(e) => Err(PersistenceError(e.to_string())),
        }
    }

    fn query_by_name(&self, name: String) -> TrainerResult<Option<Exercise>> {
        let mut stmt = self.conn.prepare(SELECT_BY_NAME).unwrap();
        let rows = stmt.query_row(&[(":name", &name)], |row| {
            let id: i64 = row.get(0).unwrap();
            let name = row.get(1).unwrap();
            let description: String = row.get(2).unwrap();
            let is_bb: bool = row.get(3).unwrap();
            let is_kb: bool = row.get(4).unwrap();

            let mut final_description = None;
            if !description.is_empty() {
                final_description = Some(description);
            }

            let mut exercise_type: ExerciseType;
            if is_bb {
                exercise_type = Barbell
            } else if is_kb {
                exercise_type = KettleBell
            } else {
                panic!("cannot determine exercise type")
            }

            Ok(Some(Exercise {
                id: Some(id),
                name,
                description: final_description,
                exercise_type: exercise_type,
            }))
        });
        match rows {
            Ok(r) => {
                Ok(r)
            },
            Err(Error::QueryReturnedNoRows) => {
                Ok(None)
            }
            Err(e) => {
                Err(QueryError(e.to_string()))
            }
        }

    }

    fn query_by_id(&self, id: i64) -> TrainerResult<Option<Exercise>> {
        todo!()
    }

    fn list(&self) -> TrainerResult<Vec<Exercise>> {
        todo!()
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
    
    use tempfile::tempdir;

    #[fixture]
    fn db_name() -> String {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        format!("testdb-{}.db", rand_string)
    }

    #[fixture]
    fn deadlift() -> Exercise {
        Exercise{
            id: None,
            name: "Deadlift".to_string(),
            description: None,
            exercise_type: Barbell,
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
    fn creation_ok(db_name: String) {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name);
        let repo = SqliteExerciseRepository::new(File(file_path.as_path())).unwrap();
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
    fn query_by_name_ok(db_name: String, deadlift: Exercise) {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name);
        let repo = SqliteExerciseRepository::new(File(file_path.as_path())).unwrap();
        let result = repo.create(&deadlift).unwrap();

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
    fn query_by_name_not_found(db_name: String) {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name);
        let repo = SqliteExerciseRepository::new(File(file_path.as_path())).unwrap();

        let result = repo.query_by_name("not_found".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[rstest]
    fn query_by_name_with_description(db_name: String) {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name);
        let repo = SqliteExerciseRepository::new(File(file_path.as_path())).unwrap();

        repo.create(&Exercise{
            id: None,
            name: "Deadlift".to_string(),
            description: Some("a lift made from a standing position, without the use of a bench or other equipment.".to_string()),
            exercise_type: ExerciseType::Barbell,
        }).unwrap();

        let query_result = repo.query_by_name("Deadlift".to_string());
        assert!(query_result.is_ok());

        let exercise = query_result.unwrap().unwrap();
        assert_eq!(exercise.id, Some(1));
        assert_eq!(exercise.name, "Deadlift");
        assert_eq!(exercise.description, Some("a lift made from a standing position, without the use of a bench or other equipment.".to_string()));
        assert_eq!(exercise.exercise_type, Barbell);
    }
}
