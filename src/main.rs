mod model;

use crate::model::{convert, Todo, User};
use axum::body::Bytes;
use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{debug_handler, middleware, Json, Router};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use futures::StreamExt;
use http_body_util::StreamBody;
use hyper::body::Frame;
use hyper::HeaderMap;
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::convert::Infallible;
use tokio::sync::mpsc;
use tokio::task_local;
use tokio_stream::wrappers::ReceiverStream;

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
    let database_url = env!("DATABASE_URL");
    let pool = PgPoolOptions::new()
        .min_connections(5)
        .max_connections(5)
        .connect(database_url)
        .await
        .unwrap();

    let result = sqlx::migrate!().run(&pool).await;

    // build our application with a route
    let state = AppState { pool };
    let app = Router::new()
        .route("/", get(fetch))
        .route("/todos", get(fetch_stream))
        // .route("/stream", get(fetch_stream2))
        .route("/todos", post(create_todos))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap()
}

async fn create_todos(
    State(state): State<AppState>,
    Json(todo_request): Json<TodoRequest>,
) -> Result<StatusCode, StatusCode> {
    let _ = Todo::insert_new_todo(&state.pool, todo_request.title)
        .await
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::CREATED)
}

#[debug_handler]
async fn fetch(State(state): State<AppState>) -> Json<Vec<Todo>> {
    println!("je suis {}", USER.get());
    Json(Todo::load(&state.pool).await)
}

task_local! {
    pub static USER: String;
}

async fn auth(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if let Some(current_user) = authorize_current_user(&state.pool, auth_header).await {
        // State is setup here in the middleware
        Ok(USER.scope(current_user, next.run(req)).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
async fn authorize_current_user(pool: &Pool<Postgres>, auth_token: &str) -> Option<String> {
    let vec = auth_token.split("Basic ").collect::<Vec<_>>();
    println!("{}", auth_token);
    println!("{}", vec.len());
    let token = vec.get(1).unwrap();
    println!("{}", token);
    if let Ok(decoded) = BASE64_STANDARD.decode(token) {
        let result = String::from_utf8(decoded).unwrap();
        let tokens = result
            .split(':')
            .map(|user| user.to_string())
            .collect::<Vec<String>>();
        let login = tokens.get(0).unwrap().clone();
        let password = tokens.get(1).unwrap().clone();
        println!("login: {}", login);
        println!("password: {}", password);
        return User::fetch(pool, &login)
            .await
            .filter(|u| u.password == password)
            .map(|u| u.login);
    }

    println!("Ops {}", auth_token);
    None
}

#[debug_handler]
async fn fetch_stream(State(state): State<AppState>) -> Result<Response<ResponseBody>, Infallible> {
    let (tx, rx) = mpsc::channel::<Data>(2000);

    // some async task
    tokio::spawn(async move {
        // some expensive operations
        let mut stream = Todo::load_stream(&state.pool).await;
        let mut i = 0;
        let mut vec = Vec::with_capacity(100);

        while let Some(message) = stream.next().await {
            let x = message.unwrap();
            vec.push(x);

            i = i + 1;
            if i % 100 == 0 {
                match tx.send(Ok(Frame::data(convert(&vec)))).await {
                    Ok(_) => {}
                    Err(err) => {
                        eprintln!("{}", err);
                        return Ok(());
                    }
                }
                vec.clear();
            }
        }
        println!("{}", i);
        tx.send(Ok(Frame::data(convert(&vec)))).await;

        // headers based off expensive operation
        let mut headers = HeaderMap::new();
        headers.append("content-type", "application/jsonlines2".parse().unwrap());
        tx.send(Ok(Frame::trailers(headers))).await.unwrap();
        Ok::<(), String>(())
    });

    let stream = ReceiverStream::new(rx);
    let body = StreamBody::new(stream);

    println!("fin");

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/jsonlines") // trailers must be declared
        .body(body)
        .unwrap())
}
/*#[debug_handler]
async fn fetch_stream2(State(state): State<AppState>) -> Result<impl IntoResponse, Infallible> {


    let body = Sse::new(Todo::load_stream_static(&state.pool).await);

    println!("fin");

    Ok(body)
}
*/
