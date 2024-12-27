/* KATA du jour
   Ecrire une API pour écrire dans la base
   Enregister des elements en base de données
   Ecrire lire de la base
*/
mod model;

use sqlx::postgres::PgPoolOptions;
use std::convert::Infallible;

use crate::model::{convert, Todo};
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{Response};
use axum::routing::{get, post};
use axum::{debug_handler, Json, Router};
use futures::StreamExt;
use http_body_util::{StreamBody};
use hyper::body::Frame;
use sqlx::{query, Pool, Postgres};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

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

async fn handler(State(state): State<AppState>, title: String) -> StatusCode {
    let result = query!("INSERT INTO todos (title, status) VALUES ($1, $2)", &title, "PENDING")
        .execute(&state.pool).await.unwrap();
    println!("{}", result.rows_affected());
    StatusCode::CREATED
}
#[debug_handler]
async fn fetch(State(state): State<AppState>) -> Json<Vec<Todo>> {
    Json(Todo::load(&state.pool).await)
}

#[debug_handler]
async fn fetch_stream(State(state): State<AppState>) -> Result<Response<ResponseBody>, Infallible> {
    let (tx, rx) = mpsc::channel::<Data>(2000);

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

        let mut i = 0;
        let mut vec = Vec::with_capacity(10);

        while let Some(message) = stream.next().await {
            let x = message.unwrap();
            vec.push(x);
            if i % 10 == 0 {
                tx.send(Ok(Frame::data(convert(&vec)))).await.unwrap();
                vec.clear();

            }
        }

        println!("{}", i);

        // headers based off expensive operation
        let mut headers = HeaderMap::new();
        headers.append("content-type", "application/jsonlines".parse().unwrap());
        tx.send(Ok(Frame::trailers(headers))).await.unwrap();
    });

    let stream = ReceiverStream::new(rx);
    let body = StreamBody::new(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/jsonlines") // trailers must be declared
        .body(body)
        .unwrap())

}
