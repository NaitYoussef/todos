mod middlewares;
mod model;
mod repository;
mod resource;
mod usecase;

use crate::middlewares::auth;
use crate::model::User;
use crate::repository::TodoAdapter;
use crate::resource::{create_todos, delete_todo, fetch, fetch_stream};
use axum::body::Bytes;
use axum::routing::{delete, get, post};
use axum::{middleware, Router};
use dotenv::dotenv;
use http_body_util::StreamBody;
use hyper::body::Frame;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::convert::Infallible;
use std::env;
use std::sync::Arc;
use tokio::{signal, task_local};
use tokio_stream::wrappers::ReceiverStream;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;

type Data = Result<Frame<Bytes>, Infallible>;
type ResponseBody = StreamBody<ReceiverStream<Data>>;

#[derive(Clone)]
struct AppState {
    pool: Pool<Postgres>,
    todo_adapter: Arc<TodoAdapter>,
}

#[derive(Deserialize)]
struct TodoRequest {
    title: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_file(true)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    dotenv().ok(); // This line loads the environment variables from the ".env" file.

    let database_url = env::var("DATABASE_URL").unwrap();
    let pool = PgPoolOptions::new()
        .min_connections(5)
        .max_connections(5)
        .connect(&database_url)
        .await
        .unwrap();

    let _result = sqlx::migrate!().run(&pool).await;

    // build our application with a route
    let state = AppState {
        pool: pool.clone(),
        todo_adapter: Arc::new(TodoAdapter::new(pool)),
    };
    let app = router(state);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap()
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(fetch))
        .route("/todos", get(fetch_stream))
        .route("/todos/{id}", delete(delete_todo))
        .route("/todos", post(create_todos))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state)
}

task_local! {
    pub static USER: User;
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    tokio::select! {_ = ctrl_c => {println!("received ctrl + C")}}
}

#[cfg(test)]
mod tests {
    use crate::repository::TodoAdapter;
    use crate::{router, AppState};
    use axum::http::StatusCode;
    use axum_test::TestServer;
    use sqlx::postgres::PgPoolOptions;
    use std::sync::Arc;
    use testcontainers_modules::postgres;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    #[sqlx::test]
    async fn api_test(){
        // Set test
        let container = postgres::Postgres::default().start().await.unwrap();
        let host_port = container.get_host_port_ipv4(5432).await.unwrap();

        let database_url = &format!(
            "postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",
        );

        let pool = PgPoolOptions::new()
            .min_connections(5)
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap();

        // Prepare data
        let _result = sqlx::migrate!().run(&pool).await;
        sqlx::query!(r#"INSERT INTO users(login, password) values('youssef', '123456')"#)
            .execute(&pool)
            .await
            .unwrap();

        let state = AppState {
            pool: pool.clone(),
            todo_adapter: Arc::new(TodoAdapter::new(pool)),
        };
        // Launch server
        let server = TestServer::new(router(state)).unwrap();

        // When
        let response = server.get("/")
            .authorization("Basic eW91c3NlZjoxMjM0NTY=")
            .await;

        // Then
        assert_eq!(response.status_code(), StatusCode::OK);
        assert_eq!(response.as_bytes(), "[]");
    }
}
