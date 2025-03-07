use crate::model::Todo;
use crate::model::TodoPort;
use crate::repository::TodoAdapter;
use crate::usecase::{cancel_todo, create_todo, TodoError};
use crate::{AppState, Data, ResponseBody, TodoRequest, USER};
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{debug_handler, Json};
use futures::StreamExt;
use http_body_util::StreamBody;
use hyper::body::Frame;
use serde::{Serialize, Serializer};
use std::convert::Infallible;
use std::ops::Deref;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, info};

#[derive(Serialize)]
pub struct TodoResourceV1 {
    id: i32,
    title: String,
    status: String,
}

impl TodoResourceV1 {
    pub fn from(todo: Todo) -> Self {
        TodoResourceV1 {
            id: todo.id(),
            title: todo.title().to_string(),
            status: todo.status().to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct ProblemDetail {
    #[serde(serialize_with = "as_u16")]
    status: StatusCode,
    title: String,
    detail: String,
}

impl ProblemDetail {
    pub fn new(status: StatusCode, detail: String) -> Self {
        ProblemDetail {
            status,
            title: status.canonical_reason().unwrap().to_string(),
            detail,
        }
    }
}
impl IntoResponse for ProblemDetail {
    fn into_response(self) -> Response {
        (self.status, Json(self)).into_response()
    }
}

fn as_u16<S>(v: &StatusCode, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_u16(v.as_u16())
}

#[debug_handler]
pub async fn fetch(State(state): State<AppState>) -> Json<Vec<TodoResourceV1>> {
    println!("Iam {}", USER.get().login);
    Json(
        TodoAdapter::load(&state.pool)
            .await
            .into_iter()
            .map(TodoResourceV1::from)
            .collect(),
    )
}

#[debug_handler]
pub async fn fetch_stream(
    State(state): State<AppState>,
) -> Result<Response<ResponseBody>, Infallible> {
    let (tx, rx) = mpsc::channel::<Data>(2000);

    // some async task
    tokio::spawn(async move {
        // some expensive operations
        let mut stream = state
            .todo_adapter
            .load_stream()
            .await
            .map(|res| res.map(TodoResourceV1::from));
        let mut i = 0;
        let mut vec = Vec::with_capacity(100);

        while let Some(message) = stream.next().await {
            let x = message.unwrap();
            vec.push(x);

            i += 1;
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
        let _ = tx.send(Ok(Frame::data(convert(&vec)))).await;

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

#[debug_handler]
pub async fn create_todos(
    State(state): State<AppState>,
    Json(todo_request): Json<TodoRequest>,
) -> Result<StatusCode, ProblemDetail> {
    let user = USER.get();
    info!(
        message = "creating todo",
        title = todo_request.title,
        user = user.login
    );
    let todo_adapter = state.todo_adapter.deref();
    create_todo(todo_adapter, todo_request.title, user.id)
        .await
        .map_err(|e| {
            error!("Error caught {:?}", e);
            ProblemDetail::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok(StatusCode::CREATED)
}

#[debug_handler]
pub async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<(), ProblemDetail> {
    cancel_todo(state.todo_adapter.deref(), id, USER.get().id)
        .await
        .map_err(|err| match err {
            TodoError::AlreadyCancel => ProblemDetail::new(
                StatusCode::BAD_REQUEST,
                String::from("only pending todos can be cancelled"),
            ),
            TodoError::NotFound => ProblemDetail::new(
                StatusCode::NOT_FOUND,
                String::from("Not found"),
            ),
            TodoError::DatabaseError => ProblemDetail::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from("problem when persisting data"),
            ),
        })
}

fn convert(value: &Vec<TodoResourceV1>) -> Bytes {
    let mut result = Vec::new();
    for todo in value {
        let mut string = serde_json::to_vec(&todo).unwrap();
        result.append(&mut string);
        let mut vec = "\n".to_string().into_bytes();
        result.append(&mut vec);
    }
    result.into()
}
