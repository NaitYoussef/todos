/* KATA du jour
   Ecrire une API pour écrire dans la base
   Enregister des elements en base de données
   Ecrire lire de la base
*/
mod model;
mod schema;

use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use diesel::Connection;
use diesel::PgConnection;
use std::env;
use crate::model::TodosToPersist;

#[tokio::main]
async fn main() {
    let database_url = "postgres://omc_projet:omc_projet@localhost:5432/todos";
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
    // build our application with a route
    let app = Router::new()
        .route("/", get(handler2))
        .route("/", post(handler));
    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler(title: String) -> StatusCode {

    StatusCode::CREATED
}
async fn handler2() -> Json<Vec<TodosToPersist>> {
    let database_url = "postgres://omc_projet:omc_projet@localhost:5432/todos";
    let mut connection = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting  to {}", database_url));

    let vec = TodosToPersist::load(&mut connection);
    Json(vec)
}
