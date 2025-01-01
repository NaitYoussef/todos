mod model;
mod repository;
mod resource;
mod middlewares;

use crate::model::{Todo, User};
use crate::repository::{convert, TodoDao, UserDao};
use crate::resource::{create_todos, delete_todo, fetch, fetch_stream, ProblemDetail, TodoResourceV1};
use axum::body::Bytes;
use axum::extract::{Path, Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post};
use axum::{debug_handler, middleware, Json, Router};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use dotenv::dotenv;
use futures::future::err;
use futures::{FutureExt, StreamExt};
use http_body_util::StreamBody;
use hyper::body::Frame;
use hyper::HeaderMap;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::convert::Infallible;
use std::env;
use std::thread::sleep;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::{signal, task_local};
use tokio::time::sleep_until;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, info};
use tracing_subscriber::fmt::format::FmtSpan;
use crate::middlewares::auth;

type Data = Result<Frame<Bytes>, Infallible>;
type ResponseBody = StreamBody<ReceiverStream<Data>>;

#[derive(Clone)]
struct AppState {
    pool: Pool<Postgres>,
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

    let result = sqlx::migrate!().run(&pool).await;

    // build our application with a route
    let state = AppState { pool };
    let app = Router::new()
        .route("/", get(fetch))
        .route("/todos", get(fetch_stream))
        .route("/todos/{id}", delete(delete_todo))
        .route("/todos", post(create_todos))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state);

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

task_local! {
    pub static USER: User;
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {_ = ctrl_c => {println!("received ctrl + C")}}
}
/*#[debug_handler]
async fn fetch_stream2(State(state): State<AppState>) -> Result<impl IntoResponse, Infallible> {


    let body = Sse::new(Todo::load_stream_static(&state.pool).await);

    println!("fin");

    Ok(body)
}
*/
