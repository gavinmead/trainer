use crate::DBType;
use api::TrainerError::{ConnectionError, ExerciseNotFound, PersistenceError, QueryError};
use api::{Exercise, TrainerResult};
use sqlx::sqlite::{SqliteConnectOptions, SqliteRow};
use sqlx::{migrate, Acquire, Error, Row, SqlitePool};
use std::str::FromStr;

#[allow(unused)]
pub struct SqlxliteExerciseRepository {
    pool: SqlitePool,
}

impl SqlxliteExerciseRepository {
    #[allow(dead_code)]
    pub async fn new(dbtype: DBType<'_>) -> TrainerResult<Self> {
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

    #[allow(dead_code)]
    pub async fn create(&self, t: &Exercise) -> TrainerResult<i64> {
        let mut conn = self.pool.acquire().await.unwrap();
        let query_result = sqlx::query(
            r#"
                INSERT INTO EXERCISE (name, description, exercise_type) VALUES (?1, ?2, ?3)
                "#,
        )
        .bind(&t.name)
        .bind(&t.description)
        .bind::<i64>(t.exercise_type.into())
        .execute(&mut *conn)
        .await;

        match query_result {
            Ok(r) => Ok(r.last_insert_rowid()),
            Err(e) => Err(PersistenceError(e.to_string())),
        }
    }

    #[allow(dead_code)]
    pub async fn query_by_id(&self, id: i64) -> TrainerResult<Option<Exercise>> {
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

    #[allow(dead_code)]
    pub async fn query_by_name(&self, name: String) -> TrainerResult<Option<Exercise>> {
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

    #[allow(dead_code)]
    pub async fn update(&self, t: &Exercise) -> TrainerResult<()> {
        let mut conn = self.pool.acquire().await.unwrap();
        let mut tx = conn.begin().await.unwrap();
        let update_result = sqlx::query(
            r#"
                UPDATE EXERCISE set name = ?1, description = ?2,
                exercise_type = ?3 WHERE id = ?4
                "#,
        )
        .bind(&t.name)
        .bind(&t.description)
        .bind::<i64>(t.exercise_type.into())
        .bind(t.id)
        .execute(&mut *tx)
        .await;

        match update_result {
            Ok(r) => {
                if r.rows_affected() == 1 {
                    let commit_result = tx.commit().await;
                    match commit_result {
                        Ok(_) => Ok(()),
                        Err(e) => Err(PersistenceError(e.to_string())),
                    }
                } else {
                    let rollback_result = tx.rollback().await;
                    match rollback_result {
                        Ok(_) => Err(ExerciseNotFound(format!(
                            "exercise {} was not found",
                            t.name.clone()
                        ))),
                        Err(e) => Err(PersistenceError(e.to_string())),
                    }
                }
            }
            Err(e) => Err(PersistenceError(e.to_string())),
        }
    }

    fn process_query(&self, r: Result<SqliteRow, Error>) -> TrainerResult<Option<Exercise>> {
        match r {
            Ok(r) => {
                println!("{:?}", r.len());
                let et: i64 = r.get(3);
                Ok(Some(Exercise {
                    id: Some(r.get(0)),
                    name: r.get(1),
                    description: r.get(2),
                    exercise_type: i64::into(et),
                }))
            }
            Err(e) => match e {
                Error::RowNotFound => Ok(None),
                _ => Err(QueryError(e.to_string())),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    use api::ExerciseType::{Barbell, KettleBell};
    use tempfile::tempdir;
    use tokio::fs;

    fn db_name() -> String {
        let rand_string: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();

        format!("testdb-{}.db3", rand_string)
    }

    fn deadlift() -> Exercise {
        Exercise {
            id: None,
            name: "Deadlift".to_string(),
            description: None,
            exercise_type: Barbell,
        }
    }

    #[tokio::test]
    async fn test_new_in_memory_connection() {
        let repo = SqlxliteExerciseRepository::new(DBType::InMemory).await;
        assert!(repo.is_ok())
    }

    #[tokio::test]
    async fn test_new_file_connection() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path())).await;
        assert!(repo.is_ok());
    }

    #[tokio::test]
    async fn test_bad_file_path() {
        let dir = tempdir().unwrap();
        let file_path = dir
            .path()
            .join("not-found")
            .join(crate::sqlx_impl::tests::db_name());
        let repo_result = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path())).await;
        assert!(repo_result.is_err());
        assert!(matches!(
            repo_result.err().unwrap(),
            ConnectionError(s) if s == "error returned from database: (code: 14) unable to open database file"
        ))
    }

    #[tokio::test]
    async fn create_ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift();
        let result = repo.create(&e).await;
        assert!(result.is_ok());
        assert!(matches!(
            result,
            Ok(i) if i > 0
        ))
    }

    #[tokio::test]
    async fn create_ok_with_description() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let mut e = deadlift();
        e.description = Some("an exercise".to_string());
        let result = repo.create(&e).await;
        assert!(result.is_ok());
        assert!(matches!(
            result,
            Ok(i) if i > 0
        ))
    }

    #[tokio::test]
    async fn create_and_get_ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift();
        let id = repo.create(&e).await.unwrap();

        let found_exercise = repo.query_by_id(id).await;
        assert!(found_exercise.is_ok());
        let ex = found_exercise.unwrap().unwrap();
        assert_eq!(ex.id, Some(id));
        assert_eq!(ex.name, ex.name);
        assert!(ex.description.is_none());
        assert_eq!(ex.exercise_type, Barbell);
    }

    #[tokio::test]
    async fn create_and_get_with_description() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let mut e = deadlift();
        e.description = Some("an exercise".to_string());
        let id = repo.create(&e).await.unwrap();

        let found_exercise = repo.query_by_id(id).await;
        assert!(found_exercise.is_ok());
        let ex = found_exercise.unwrap().unwrap();
        assert_eq!(ex.id, Some(id));
        assert_eq!(ex.name, ex.name);
        assert_eq!(ex.description.unwrap(), "an exercise".to_string());
        assert_eq!(ex.exercise_type, Barbell);
    }

    #[tokio::test]
    async fn query_id_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let found_exercise = repo.query_by_id(100).await;
        assert!(found_exercise.is_ok());
        assert!(found_exercise.unwrap().is_none());
    }

    #[tokio::test]
    async fn query_by_name_ok() {
        let queries = vec!["Deadlift", "deadlift", "DeadLift", "DEADLIFT", "dEaDlIfT"];

        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift();
        let _ = repo.create(&e).await.unwrap();

        for q in queries {
            let query_result = repo.query_by_name(q.to_string()).await;
            assert!(query_result.is_ok());

            let exercise = query_result.unwrap().unwrap();
            assert_eq!(exercise.id, Some(1));
            assert_eq!(exercise.name, "Deadlift");
            assert_eq!(exercise.description, None);
            assert_eq!(exercise.exercise_type, Barbell);
        }
    }

    #[tokio::test]
    async fn query_by_name_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();
        let query_result = repo.query_by_name("not-found".to_string()).await;
        assert!(query_result.is_ok());
        assert!(query_result.unwrap().is_none())
    }

    #[tokio::test]
    async fn update_ok() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift();
        let id = repo.create(&e).await.unwrap();

        let mut found_ex = repo.query_by_id(id).await.unwrap().unwrap();
        found_ex.description = Some("updated description".to_string());
        found_ex.exercise_type = KettleBell;
        found_ex.name = "DL".to_string();

        let update_result = repo.update(&found_ex).await;
        assert!(update_result.is_ok());

        let found_ex = repo.query_by_id(id).await.unwrap().unwrap();
        assert_eq!(found_ex.name, "DL".to_string());
        assert_eq!(found_ex.exercise_type, KettleBell);
        assert_eq!(
            found_ex.description,
            Some("updated description".to_string())
        );
    }

    #[tokio::test]
    async fn update_not_found() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift();
        let update_result = repo.update(&e).await;
        assert!(update_result.is_err());
        assert!(matches!(update_result.err().unwrap(),
            ExerciseNotFound(s) if s == "exercise Deadlift was not found".to_string()));
    }

    #[tokio::test]
    async fn create_failed() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        //Remove teh db file to test failure modes
        fs::remove_file(file_path.as_path()).await.unwrap();
        let e = deadlift();
        let id = repo.create(&e).await;
        assert!(id.is_err());
        assert!(matches!(id.err().unwrap(), PersistenceError(_)))
    }

    #[tokio::test]
    async fn create_duplicate_name() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(crate::sqlx_impl::tests::db_name());
        let repo = SqlxliteExerciseRepository::new(DBType::File(file_path.as_path()))
            .await
            .unwrap();

        let e = deadlift();
        let _ = repo.create(&e).await;

        let same_ex = deadlift();
        let result = repo.create(&same_ex).await;
        assert!(result.is_err());
        assert!(matches!(result.err().unwrap(), PersistenceError(_)))
    }
}
