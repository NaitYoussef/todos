use strum_macros::{Display, EnumString};

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
    pub id: i32,
    pub title: String,
    pub status: Status,
}

impl Todo {
    pub fn new(id: i32, title: String, status: Status) -> Todo {
        Todo {
            id,
            title,
            status,
        }
    }
}

#[derive(Display, EnumString)]
pub enum Status {
    Active,
    Pending,
    Cancelled,
}
