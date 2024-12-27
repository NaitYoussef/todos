use axum::body::{Body, Bytes};
use axum::response::{IntoResponse, Sse};
use futures::stream::{self};
use serde::Serialize;
use sqlx::{query, Error, Pool, Postgres};
use std::iter::Map;
use tokio_stream::Stream;

impl Todo {
    pub fn new(title: String, status: String) -> Self {
        Self { title, status }
    }

    pub async fn load(pool: &Pool<Postgres>) -> Vec<Self> {
        let query = query!(r#"SELECT status, title, id FROM todos"#)
            .fetch_all(pool)
            .await
            .unwrap();

        query
            .into_iter()
            .map(|row| Todo::new(row.title, row.status))
            .collect()
    }

    /*    pub async fn load_stream(pool: Pool<Postgres>) -> impl Stream {
            query!(r#"SELECT status, title, id FROM todos"#)
                .fetch(&pool)
                .map(|row| match row {
                    Ok(todo) => Ok(Todo::new(todo.title, todo.status)),
                    Err(_) => Err("error)"),
                })
        }
    */
}

#[derive(Serialize)]
pub struct Todo {
    title: String,
    status: String,
}

pub enum Status {
    Active,
    Pending,
    Cancelled,
}

impl From<Todo> for Bytes {
    fn from(value: Todo) -> Self {
        let mut string = serde_json::to_vec(&value).unwrap();
        let mut vec = "\n".to_string().into_bytes();
        string.append(&mut vec);
        string.into()
    }
}
pub fn convert(value: &Vec<Todo>) -> Bytes {
    let mut result = Vec::new();
    for todo in value {
        let mut string = serde_json::to_vec(&todo).unwrap();
        result.append(&mut string);
        let mut vec = "\n".to_string().into_bytes();
        result.append(&mut vec);
    }
    result.into()
}
