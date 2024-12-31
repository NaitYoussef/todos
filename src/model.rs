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
        Todo { id, title, status }
    }

    pub fn cancel(&mut self) -> bool {
        if self.status != Status::Pending {
            return false;
        }
        self.status = Status::Cancelled;
        true
    }
}

#[derive(Display, EnumString, PartialEq)]
pub enum Status {
    Active,
    Pending,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn name() {
        let mut todo = Todo::new(1, String::from("TOTO"), Status::Pending);
    }
}
