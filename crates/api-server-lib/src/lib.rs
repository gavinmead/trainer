use axum::routing::get;
use axum::Router;

#[allow(dead_code)]
struct CreateExerciseRequest {
    name: String,
    description: Option<String>,
    exercise_type: String,
}

#[allow(dead_code)]
struct CreateExerciseResponse {
    id: i64,
}

#[allow(dead_code)]
struct GetExerciseResponse {
    id: i64,
    name: String,
    description: Option<String>,
    exercise_type: String,
}

#[allow(dead_code)]
fn app() -> Router {
    Router::new().route("/", get(|| async { "Hello, World!" }))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tokio::net::TcpListener;
    use tower::ServiceExt;
    // for `collect`
    use super::*;

    #[tokio::test]
    async fn hello_world() {
        let app = app();

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

    #[tokio::test]
    async fn the_real_deal() {
        let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app()).await.unwrap();
        });

        let client =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
                .build_http();

        let response = client
            .request(
                Request::builder()
                    .uri(format!("http://{addr}"))
                    .header("Host", "localhost")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"Hello, World!");
    }
}
