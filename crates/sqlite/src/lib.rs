use api::exercise::{Exercise, ExerciseRepository};
use api::RepositoryError::{ConnectionError, ItemNotFoundError, QueryError};
use api::{RepositoryError, RepositoryResult};
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqliteRow};
use sqlx::{migrate, Acquire, Error, Row, SqlitePool};
use std::path::Path;
use std::str::FromStr;
use tracing::instrument;

#[derive(Clone, Debug)]
pub enum DBType<'a> {
    InMemory,
    File(&'a Path),
}

#[derive(Clone, Debug)]
pub struct SqliteExerciseRepository {
    pool: SqlitePool,
}

impl SqliteExerciseRepository {
    #[instrument]
    pub async fn new(dbtype: DBType<'_>) -> RepositoryResult<Self> {
        let pool_result: Result<SqlitePool, Error> = match dbtype {
            DBType::InMemory => SqlitePool::connect("sqlite::memory:").await,
            DBType::File(f) => {
                let opts = SqliteConnectOptions::from_str(
                    format!("sqlite://{}", f.to_str().unwrap()).as_str(),
                )
                .unwrap()
                .create_if_missing(true)
                .foreign_keys(true);

                SqlitePool::connect_with(opts).await
            }
        };

        match pool_result {
            Ok(p) => {
                let migrate_result = migrate!("db/migrations/exercises").run(&p).await;

                match migrate_result {
                    Ok(_) => Ok(Self { pool: p }),
                    Err(e) => Err(ConnectionError(e.to_string())),
                }
            }
            Err(e) => Err(ConnectionError(e.to_string())),
        }
    }

    fn process_query(&self, r: Result<SqliteRow, Error>) -> RepositoryResult<Exercise> {
        match r {
            Ok(r) => {
                println!("{:?}", r.len());
                let et: i64 = r.get(3);
                Ok(Exercise {
                    id: Some(r.get(0)),
                    name: r.get(1),
                    description: r.get(2),
                    exercise_type: i64::into(et),
                })
            }
            Err(e) => match e {
                Error::RowNotFound => Err(RepositoryError::ItemNotFoundError),
                _ => Err(RepositoryError::QueryError(e.to_string())),
            },
        }
    }
}

#[async_trait]
impl ExerciseRepository for SqliteExerciseRepository {
    #[instrument(skip(self), fields(name = exercise.name))]
    async fn create(&self, exercise: &Exercise) -> RepositoryResult<i64> {
        let mut conn = self.pool.acquire().await.unwrap();
        let query_result = sqlx::query(
            r#"
                INSERT INTO EXERCISE (name, description, exercise_type) VALUES (?1, ?2, ?3)
                "#,
        )
        .bind(&exercise.name)
        .bind(&exercise.description)
        .bind::<i64>(exercise.exercise_type.into())
        .execute(&mut *conn)
        .await;

        match query_result {
            Ok(r) => Ok(r.last_insert_rowid()),
            Err(e) => Err(RepositoryError::PersistenceError(e.to_string())),
        }
    }

    #[instrument(skip(self), fields(name = exercise.name))]
    async fn update(&self, exercise: &Exercise) -> RepositoryResult<()> {
        let mut conn = self.pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();
        let update_result = sqlx::query(
            r#"
                UPDATE EXERCISE set name = ?1, description = ?2,
                exercise_type = ?3 WHERE id = ?4
                "#,
        )
        .bind(&exercise.name)
        .bind(&exercise.description)
        .bind::<i64>(exercise.exercise_type.into())
        .bind(exercise.id)
        .execute(&mut *tx)
        .await;

        match update_result {
            Ok(r) => {
                if r.rows_affected() == 1 {
                    let commit_result = tx.commit().await;
                    match commit_result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(RepositoryError::PersistenceError(e.to_string())),
                    }
                } else {
                    let rollback_result = tx.rollback().await;
                    match rollback_result {
                        Ok(_) => Err(RepositoryError::ItemNotFoundError),
                        Err(e) => Err(RepositoryError::PersistenceError(e.to_string())),
                    }
                }
            }
            Err(e) => Err(RepositoryError::PersistenceError(e.to_string())),
        }
    }

    #[instrument(skip(self), fields(name = name))]
    async fn query_by_name(&self, name: String) -> RepositoryResult<Exercise> {
        let mut conn = self.pool.acquire().await.unwrap();
        let query_result = sqlx::query(
            r#"
                SELECT id, name, description, exercise_type
                FROM EXERCISE WHERE deleted = 0 AND
                name = ?1 COLLATE NOCASE
                "#,
        )
        .bind(name)
        .fetch_one(&mut *conn)
        .await;

        self.process_query(query_result)
    }

    #[instrument(skip(self), fields(id))]
    async fn query_by_id(&self, id: i64) -> RepositoryResult<Exercise> {
        let mut conn = self.pool.acquire().await.unwrap();
        let query_result = sqlx::query(
            r#"
                SELECT id, name, description, exercise_type
                FROM EXERCISE WHERE id = ?1 AND deleted = 0
                "#,
        )
        .bind(id)
        .fetch_one(&mut *conn)
        .await;

        self.process_query(query_result)
    }

    #[instrument(skip(self))]
    async fn list(&self) -> RepositoryResult<Vec<Exercise>> {
        let mut conn = self.pool.acquire().await.unwrap();
        let query_result = sqlx::query(
            r#"
            SELECT id, name, description, exercise_type FROM
            EXERCISE WHERE DELETED = 0;
            "#,
        )
        .fetch_all(&mut *conn)
        .await;
        match query_result {
            Ok(rows) => {
                let mut exercises: Vec<Exercise> = vec![];
                for row in rows {
                    let r = self.process_query(Ok(row)).unwrap();
                    exercises.push(r)
                }
                Ok(exercises)
            }
            Err(err) => Err(QueryError(err.to_string())),
        }
    }

    #[instrument(skip(self), fields(id))]
    async fn delete(&self, id: i64) -> RepositoryResult<()> {
        let mut conn = self.pool.acquire().await.unwrap();
        let update_result = sqlx::query(
            r#"
            UPDATE EXERCISE SET deleted = 1 WHERE id = ?1
        "#,
        )
        .bind(id)
        .execute(&mut *conn)
        .await;
        match update_result {
            Ok(result) => match result.rows_affected() {
                0 => Err(ItemNotFoundError),
                1 => Ok(()),
                _ => panic!("more than one row was updated which should be impossible"),
            },
            Err(err) => Err(RepositoryError::DeleteError(err.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    use api::exercise::ExerciseType::{Barbell, KettleBell};
    use api::RepositoryError::{ConnectionError, PersistenceError};
    use tempfile::tempdir;
    use test_log::test;
    use tokio::fs;

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
    async fn test_new_in_memory_connection() {
        let repo = SqliteExerciseRepository::new(DBType::InMemory).await;
        assert!(repo.is_ok())
    }

    #[test(tokio::test)]
    async fn test_new_file_connection() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path())).await;
        assert!(repo.is_ok());
    }

    #[test(tokio::test)]
    async fn test_bad_file_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("not-found").join(db_name());
        let repo_result = SqliteExerciseRepository::new(DBType::File(file_path.as_path())).await;
        assert!(repo_result.is_err());
        assert!(matches!(
            repo_result.err().unwrap(),
            ConnectionError(s) if s == "error returned from database: (code: 14) unable to open database file"
        ))
    }

    #[test(tokio::test)]
    async fn create_ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift(None);
        let result = repo.create(&e).await;
        assert!(result.is_ok());
        assert!(matches!(
            result,
            Ok(i) if i > 0
        ))
    }

    #[test(tokio::test)]
    async fn create_ok_with_description() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let mut e = deadlift(None);
        e.description = Some("an exercise".to_string());
        let result = repo.create(&e).await;
        assert!(result.is_ok());
        assert!(matches!(
            result,
            Ok(i) if i > 0
        ))
    }

    #[test(tokio::test)]
    async fn create_and_get_ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift(None);
        let id = repo.create(&e).await.unwrap();

        let found_exercise = repo.query_by_id(id).await;
        assert!(found_exercise.is_ok());
        let ex = found_exercise.unwrap();
        assert_eq!(ex.id, Some(id));
        assert_eq!(ex.name, ex.name);
        assert!(ex.description.is_none());
        assert_eq!(ex.exercise_type, Barbell);
    }

    #[test(tokio::test)]
    async fn create_and_get_with_description() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let mut e = deadlift(None);
        e.description = Some("an exercise".to_string());
        let id = repo.create(&e).await.unwrap();

        let found_exercise = repo.query_by_id(id).await;
        assert!(found_exercise.is_ok());
        let ex = found_exercise.unwrap();
        assert_eq!(ex.id, Some(id));
        assert_eq!(ex.name, ex.name);
        assert_eq!(ex.description.unwrap(), "an exercise".to_string());
        assert_eq!(ex.exercise_type, Barbell);
    }

    #[test(tokio::test)]
    async fn query_id_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let found_exercise = repo.query_by_id(100).await;
        assert!(found_exercise.is_err());
        assert!(matches!(found_exercise.err().unwrap(), ItemNotFoundError))
    }

    #[test(tokio::test)]
    async fn query_by_name_ok() {
        let queries = vec!["Deadlift", "deadlift", "DeadLift", "DEADLIFT", "dEaDlIfT"];

        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift(None);
        let _ = repo.create(&e).await.unwrap();

        for q in queries {
            let query_result = repo.query_by_name(q.to_string()).await;
            assert!(query_result.is_ok());

            let exercise = query_result.unwrap();
            assert_eq!(exercise.id, Some(1));
            assert_eq!(exercise.name, "Deadlift");
            assert_eq!(exercise.description, None);
            assert_eq!(exercise.exercise_type, Barbell);
        }
    }

    #[test(tokio::test)]
    async fn query_by_name_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();
        let query_result = repo.query_by_name("not-found".to_string()).await;
        assert!(query_result.is_err());
        assert!(matches!(query_result.err().unwrap(), ItemNotFoundError))
    }

    #[test(tokio::test)]
    async fn update_ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift(None);
        let id = repo.create(&e).await.unwrap();

        let mut found_ex = repo.query_by_id(id).await.unwrap();
        found_ex.description = Some("updated description".to_string());
        found_ex.exercise_type = KettleBell;
        found_ex.name = "DL".to_string();

        let update_result = repo.update(&found_ex).await;
        assert!(update_result.is_ok());

        let found_ex = repo.query_by_id(id).await.unwrap();
        assert_eq!(found_ex.name, "DL".to_string());
        assert_eq!(found_ex.exercise_type, KettleBell);
        assert_eq!(
            found_ex.description,
            Some("updated description".to_string())
        );
    }

    #[test(tokio::test)]
    async fn update_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift(None);
        let update_result = repo.update(&e).await;
        assert!(update_result.is_err());
        assert!(matches!(
            update_result.err().unwrap(),
            RepositoryError::ItemNotFoundError
        ));
    }

    #[test(tokio::test)]
    async fn create_failed() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        //Remove teh db file to test failure modes
        fs::remove_file(file_path.as_path()).await.unwrap();
        let e = deadlift(None);
        let id = repo.create(&e).await;
        assert!(id.is_err());
        assert!(matches!(id.err().unwrap(), PersistenceError(_)))
    }

    #[test(tokio::test)]
    async fn create_duplicate_name() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift(None);
        let _ = repo.create(&e).await;

        let same_ex = deadlift(None);
        let result = repo.create(&same_ex).await;
        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), PersistenceError(_)))
    }

    #[test(tokio::test)]
    async fn list_ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let dl = deadlift(None);
        let bp = benchpress(None);
        let sq = squat(None);

        repo.create(&dl).await.unwrap();
        repo.create(&bp).await.unwrap();
        repo.create(&sq).await.unwrap();

        let list_result = repo.list().await;
        assert!(list_result.is_ok());

        let exercises = list_result.unwrap();
        assert_eq!(3, exercises.len());
    }

    #[test(tokio::test)]
    async fn list_ok_no_deleted_items() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let dl = deadlift(None);
        let bp = benchpress(None);
        let sq = squat(None);

        repo.create(&dl).await.unwrap();
        repo.create(&bp).await.unwrap();
        let id = repo.create(&sq).await.unwrap();
        repo.delete(id).await.unwrap();

        let list_result = repo.list().await;
        assert!(list_result.is_ok());

        let exercises = list_result.unwrap();
        assert_eq!(2, exercises.len());
    }

    #[test(tokio::test)]
    async fn delete_ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();
        let dl = deadlift(None);
        let id = repo.create(&dl).await.unwrap();
        let delete_result = repo.delete(id.clone()).await;
        assert!(delete_result.is_ok());

        //Make sure the items is not returned
        let query_result = repo.query_by_id(id).await;
        assert!(query_result.is_err());
        assert!(matches!(query_result.err().unwrap(), ItemNotFoundError,))
    }

    #[test(tokio::test)]
    async fn delete_item_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(db_name());
        let repo = SqliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let delete_result = repo.delete(1000).await;
        assert!(delete_result.is_err());
        assert!(matches!(delete_result.err().unwrap(), ItemNotFoundError,))
    }
}
