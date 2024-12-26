use serde::Serialize;
impl Todo {
    pub fn new(title: String, status: String) -> Self {
        Self { title, status }
    }
}

#[derive(Serialize)]
pub struct Todo {
    title: String,
    status: String,
}

struct TodoRepository {

}

pub enum Status {
    Active,
    Pending,
    Cancelled,
}
