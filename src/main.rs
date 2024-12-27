/* KATA du jour
   Ecrire une API pour écrire dans la base
   Enregister des elements en base de données
   Ecrire lire de la base
*/
mod model;

use sqlx::postgres::PgPoolOptions;
use std::convert::Infallible;

use crate::model::Todo;
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{debug_handler, Json, Router};
use http_body_util::{BodyExt, StreamBody};
use hyper::body::Frame;
use sqlx::{query, Executor, Pool, Postgres};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures::StreamExt;

type Data = Result<Frame<Bytes>, Infallible>;
type ResponseBody = StreamBody<ReceiverStream<Data>>;

#[derive(Clone)]
struct AppState {
    pool: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    let database_url = "postgres://omc_projet:omc_projet@localhost:5432/todos";
    let pool = PgPoolOptions::new()
        .min_connections(50)
        .max_connections(50)
        .connect(database_url)
        .await
        .unwrap();
    // build our application with a route

    let state = AppState { pool };
    let app = Router::new()
        .route("/", get(fetch))
        .route("/todos", get(fetch_stream))
        .route("/", post(handler))
        .with_state(state);
    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap()
}

async fn handler(title: String) -> StatusCode {
    StatusCode::CREATED
}
#[debug_handler]
async fn fetch(State(state): State<AppState>) -> Json<Vec<Todo>> {
    Json(Todo::load(&state.pool).await)
}

#[debug_handler]
async fn fetch_stream(State(state): State<AppState>) -> Result<Response<ResponseBody>, Infallible> {
    let (tx, rx) = mpsc::channel::<Data>(2);

    // some async task
    tokio::spawn(async move {
        // some expensive operations

       // let steam = Todo::load_stream(state.pool.clone()).await;
        let mut stream = query!(r#"SELECT status, title, id FROM todos"#)
            .fetch(&state.pool)
            .map(|row| match row {
                Ok(todo) => Ok(Todo::new(todo.title, todo.status)),
                Err(_) => Err("error)"),
            });

        while let Some(message) = stream.next().await {
            tx.send(Ok(Frame::data(Bytes::from(message.unwrap()))))
                .await.unwrap();
        }

        // headers based off expensive operation
        let mut headers = HeaderMap::new();
        tx.send(Ok(Frame::trailers(headers))).await.unwrap();
    });

    let stream = ReceiverStream::new(rx);
    let body = StreamBody::new(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Trailer", "chunky-trailer") // trailers must be declared
        .body(body)
        .unwrap())

}
