use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Json, Router};
use axum::http::StatusCode;
use axum::routing::{get, post};
use api::exercise::{Exercise, ExerciseManagement, ExerciseManager, ExerciseRepository};
use sqlite::SqliteExerciseRepository;
use serde::{Deserialize, Serialize};
use api::TrainerResult;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateExerciseRequest {
    name: String,
    description: Option<String>,
    exercise_type: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateExerciseResponse {
    id: i64
}

#[derive(Clone)]
struct ExerciseApiState<'a> {
    mgr: &'a ExerciseManager<'a, SqliteExerciseRepository>,
}

impl<'a> ExerciseApiState<'a> {
    fn new(mgr: &'a ExerciseManager<SqliteExerciseRepository>) -> Self {
        Self {
            mgr
        }
    }
}

async fn create_exercise(State(exercise_api_state): State<ExerciseApiState<'_>>, Json(req): Json<CreateExerciseRequest>) -> Result<CreateExerciseResponse, ()>{
    let mut exercise = Exercise{
        id: None,
        name: req.name,
        description: req.description,
        exercise_type: req.exercise_type.into(),
    };
    let result = exercise_api_state.mgr.save(&mut exercise).await;
    match result {
        Ok(_) => {
            (StatusCode::OK, Json(CreateExerciseResponse{
                id: exercise.id.unwrap(),
            }))
        }
        Err(_) => {
            (StatusCode::INTERNAL_SERVER_ERROR, Json(()))
        }
    }
}

fn exercise_api() -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/exercises", post(create_exercise))
}

#[cfg(test)]
mod tests {
    use std::os::macos::raw::stat;
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use http_body_util::BodyExt; // for `collect`
    use tower::{ServiceExt};
    use sqlite::{DBType, SqliteExerciseRepository};
    // for `call`, `oneshot`, and `ready`

    #[tokio::test]
    async fn hello_world() {
        let repo: SqliteExerciseRepository  = SqliteExerciseRepository::new(DBType::InMemory).await.unwrap();
        let mgr: ExerciseManager<SqliteExerciseRepository> = ExerciseManager::new(&repo).unwrap();
        let state: ExerciseApiState = ExerciseApiState::new(&mgr);
        let app = exercise_api();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Hello, World!");
    }
}