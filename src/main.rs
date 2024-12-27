/* KATA du jour
   Ecrire une API pour écrire dans la base
   Enregister des elements en base de données
   Ecrire lire de la base
*/
mod model;

use crate::model::TodosToPersist;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

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
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://omc_projet:omc_projet@localhost:5432/todos")
        .await
        .unwrap();
    let state = AppState { pool };
    // build our application with a route
    let app = Router::new()
        .route("/", get(fetch_all))
        .route("/", post(handler))
        .with_state(state);
    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(State(state): State<AppState>, Json(title): Json<TodoRequest>) -> StatusCode {
    TodosToPersist::create_new(state.pool, title.title).await;
    StatusCode::CREATED
}

async fn fetch_all(State(state): State<AppState>) -> Json<Vec<TodosToPersist>> {
    let pool = state.pool;

    Json(TodosToPersist::load(pool).await)
}

