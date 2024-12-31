use serde::Serialize;
use crate::model::Todo;

#[derive(Serialize)]
pub struct TodoResourceV1 {
    id: i32,
    title: String,
    status: String,
}

impl TodoResourceV1 {
    pub fn from(todo: Todo) -> Self {
        TodoResourceV1 {
            id: todo.id,
            title: todo.title,
            status: todo.status.to_string(),
        }
    }
}