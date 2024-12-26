/* KATA du jour
   Ecrire une API pour écrire dans la base
   Enregister des elements en base de données
   Ecrire lire de la base
*/
mod model;

use sqlx::postgres::PgPoolOptions;

use crate::model::Todo;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{debug_handler, Json, Router};
use sqlx::{Executor, Pool, Postgres};

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
