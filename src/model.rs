use serde::Serialize;
use sqlx::{query, Pool, Postgres};

impl Todo {
    pub fn new(title: String, status: String) -> Self {
        Self { title, status }
    }

    pub async fn load(pool: &Pool<Postgres>) -> Vec<Self>{
        let query = query!(r#"SELECT status, title, id FROM todos"#)
            .fetch_all(pool)
            .await.unwrap();

        query.into_iter().map(|row| Todo::new(row.title, row.status)).collect()
    }
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
