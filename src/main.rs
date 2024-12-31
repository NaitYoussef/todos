mod model;
mod repository;
mod resource;

use crate::model::{Todo, User};
use crate::repository::{convert, TodoDao, UserDao};
use crate::resource::{ProblemDetail, TodoResourceV1};
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
use tokio::sync::mpsc;
use tokio::task_local;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, info};
use tracing_subscriber::fmt::format::FmtSpan;

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
        .route("/todos/:id", delete(delete_todo))
        .route("/todos", post(create_todos))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());
    info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap()
}

async fn create_todos(
    State(state): State<AppState>,
    Json(todo_request): Json<TodoRequest>,
) -> Result<StatusCode, ProblemDetail> {
    let user = USER.get();
    info!(
        message = "creating todo",
        title = todo_request.title,
        user = user.login
    );
    let _ = TodoDao::insert_new_todo(&state.pool, todo_request.title, user.id)
        .await
        .map_err(|e| {
            error!("Error caught {:?}", e);
            ProblemDetail::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    Ok(StatusCode::CREATED)
}

#[debug_handler]
async fn fetch(State(state): State<AppState>) -> Json<Vec<TodoResourceV1>> {
    println!("je suis {}", USER.get().login);
    Json(
        TodoDao::load(&state.pool)
            .await
            .into_iter()
            .map(|todo| TodoResourceV1::from(todo))
            .collect(),
    )
}

task_local! {
    pub static USER: User;
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
async fn authorize_current_user(pool: &Pool<Postgres>, auth_token: &str) -> Option<User> {
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
        return UserDao::fetch(pool, &login)
            .await
            .filter(|u| u.password == password);
    }

    println!("Ops {}", auth_token);
    None
}

async fn delete_todo(State(state): State<AppState>, Path(id): Path<i32>) -> Result<(), ProblemDetail> {
    let optionalTodo = TodoDao::load_by_id(&state.pool, id).await;
    if let Some(mut todo) = optionalTodo {
        if !todo.cancel() {
            return Err(ProblemDetail::new(StatusCode::BAD_REQUEST, String::from("only pending todos can be cancelled")))
        }
        TodoDao::cancel(todo.id, &state.pool).await;
        return Ok(());
    }
    Err(ProblemDetail::new(StatusCode::NOT_FOUND, String::from("Not found")))
}

#[debug_handler]
async fn fetch_stream(State(state): State<AppState>) -> Result<Response<ResponseBody>, Infallible> {
    let (tx, rx) = mpsc::channel::<Data>(2000);

    // some async task
    tokio::spawn(async move {
        // some expensive operations
        let mut stream = TodoDao::load_stream(&state.pool)
            .await
            .map(|res| res.map(|todo| TodoResourceV1::from(todo)));
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
