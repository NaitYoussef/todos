/* KATA du jour
   Ecrire une API pour écrire dans la base
   Enregister des elements en base de données
   Ecrire lire de la base
*/
mod model;

use sqlx::postgres::PgPoolOptions;

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router, debug_handler};
use axum::extract::State;
use sqlx::{query, Executor, Pool, Postgres};
use crate::model::Todo;

#[derive(Clone)]
struct AppState {
    pool: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    let database_url = "postgres://omc_projet:omc_projet@localhost:5432/todos";
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url).await.unwrap();
    // build our application with a route

    let state = AppState { pool };
    let app = Router::new()
        .route("/", get(handler2))
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
async fn handler2(State(state): State<AppState>) -> Json<Vec<Todo>> {
    let query = query!(r#"SELECT status, title, id FROM todos WHERE id = $1"#, 1)
        .fetch_one(&state.pool)
        .await.unwrap();

    let todo = Todo::new(query.title, query.status);

    Json(vec![todo])
}
