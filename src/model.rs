/*impl Todo12 {
    pub fn new(title: String, status: Status) -> Self {

    }
}

pub struct Todo12 {
    title: String,
    status: Status,
}*/

pub struct TodosToPersist {
    pub id: i32,
    pub title: String,
    status: String,
}

pub enum Status {
    Active,
    Pending,
    Cancelled,
}
