/*impl Todo12 {
    pub fn new(title: String, status: Status) -> Self {

    }
}

pub struct Todo12 {
    title: String,
    status: Status,
}*/
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use tracing::info;

#[derive(Debug, Serialize, FromRow)]
pub struct TodosToPersist {
    id: i32,
    title: String,
    status: String,
}

impl TodosToPersist {
    pub fn new(id: i32, title: String, status: String) -> Self {
        Self { id, title, status }
    }

    pub async fn load(pool: Pool<Postgres>) -> Vec<TodosToPersist> {
        sqlx::query_as!(TodosToPersist, r#"select * from todos"#)
            .fetch_all(&pool)
            .await
            .unwrap()
    }

    pub async fn create_new(pool: Pool<Postgres>, title: String) {
        info!("creating new todos with title {}", title = title);
        let mut transaction = pool.begin().await.unwrap();
        let result = sqlx::query!(r#"INSERT INTO todos (title, status) VALUES ($1, $2)"#, title, "PENDING")
            .execute(&mut *transaction).await.unwrap();
        transaction.commit().await.unwrap();
    }

}

pub enum Status {
    Active,
    Pending,
    Cancelled,
}
