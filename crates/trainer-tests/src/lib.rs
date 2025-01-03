use async_trait::async_trait;

#[derive(Clone, Debug)]
enum Err {
    E
}

type MyResult<T> = Result<T, Err>;

#[derive(Clone, Debug)]
struct Exercise {
    id: i64,
    name: String
}

#[async_trait]
trait ExerciseRepository {
    async fn create(&self, e: &Exercise) -> MyResult<i64>;
}

#[derive(Clone, Debug)]
struct MyRepo<'a> {
    s: &'a String
}

impl<'a> MyRepo<'a> {
    fn new(s: &'a String) -> MyRepo<'a> {
        MyRepo{
            s: &s
        }
    }
}

#[async_trait]
impl ExerciseRepository for MyRepo<'_> {
    async fn create(&self, e: &Exercise) -> MyResult<i64> {
        println!("Repo {}: e: {}:{}", self.s, e.id.clone(), e.name);
        Ok(e.id.clone())
    }
}


#[async_trait]
trait ExerciseManager {
    async fn save(&self, e: &Exercise) -> MyResult<i64>;
}

#[derive(Clone)]
struct MyExerciseManager<'a, T: ExerciseRepository> {
    repo: &'a T
}

impl <'a, T: ExerciseRepository> MyExerciseManager<'a, T> {
    fn new(t: &'a T) -> Self {
        Self {
            repo: &t
        }
    }
}

#[async_trait]
impl <T: ExerciseRepository + Sync> ExerciseManager for MyExerciseManager<'_, T> {
    async fn save(&self, e: &Exercise) -> MyResult<i64> {
         self.repo.create(e).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ok() {
        let binding = "foo".to_string();
        let b2 = "bar".to_string();
        let repo = MyRepo::new(&binding);
        let repo2 = MyRepo::new(&b2);

        let mgr = MyExerciseManager::new(&repo);
        let e = Exercise{
            id: 1,
            name: "Test".to_string()
        };
        repo2.create(&e.clone()).await;
        let result = mgr.save(&e).await;
        assert!(result.is_ok())
    }
}