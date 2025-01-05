use std::error::Error;
use futures::Stream;
use strum_macros::{Display, EnumString};

#[cfg(test)]
use mockall::automock;

#[derive(Clone)]
pub struct User {
    pub id: i32,
    pub login: String,
    pub password: String,
}

impl User {
    pub fn new(id: i32, login: String, password: String) -> User {
        User {
            id,
            login,
            password,
        }
    }
}

pub struct Todo {
    id: i32,
    title: String,
    status: Status,
}

impl Todo {
    pub fn new(id: i32, title: String, status: Status) -> Todo {
        Todo { id, title, status }
    }

    pub fn cancel(&mut self) -> bool {
        if self.status != Status::Pending {
            return false;
        }
        self.status = Status::Cancelled;
        true
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn status(&self) -> &Status {
        &self.status
    }
}

#[cfg_attr(test, automock)]
pub trait TodoPort {
    async fn load_by_id(&self, id: i32) -> Option<Todo>;
    async fn insert_new_todo(
        &self,
        title: String,
        user_id: i32,
    ) -> Result<Todo, Box<dyn Error>>;

    async fn cancel(&self, id: i32) -> Result<(), String>;

    async fn load_stream(
        &self,
    ) -> impl Stream<Item = Result<Todo, String>>;
}

#[derive(Display, EnumString, PartialEq)]
pub enum Status {
    Active,
    Pending,
    Cancelled,
}
