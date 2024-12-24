impl Todo {
    pub fn new(title: String, status: Status) -> Self {
        Todo { title, status }
    }
}

pub struct Todo {
    title: String,
    status: Status,
}

pub enum Status {
    Active,
    Pending,
    Cancelled,
}
