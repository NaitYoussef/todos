use crate::model::{Status, Todo, TodoPort, User};
use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt};
use sqlx::{query, query_as, FromRow, Pool, Postgres};
use std::error::Error;
use std::str::FromStr;
use tokio_stream::Stream;
use tracing::info;

// TodoDao logic
pub struct TodoAdapter {
    pool: Pool<Postgres>,
}
#[derive(FromRow)]
pub struct TodoDao {
    id: i32,
    title: String,
    status: String,
    user_id: i32,
    created_at: DateTime<Utc>,
}

impl TodoPort for TodoAdapter {
    async fn load_by_id(&self, id: i32) -> Option<Todo> {
        query_as!(
            TodoDao,
            r#"SELECT id, status, title, user_id, created_at FROM todos WHERE id=$1"#,
            id
        )
        .fetch_one(&self.pool)
        .await
        .ok()
        .map(Todo::from)
    }

    async fn insert_new_todo(&self, title: String, user_id: i32) -> Result<Todo, Box<dyn Error>> {
        let result = query!(
            r#"INSERT INTO todos (status, title, user_id, created_at) VALUES ('Pending', $1, $2, $3) RETURNING ID"#,
            title,
            user_id,
            Utc::now()
        ).fetch_one(&self.pool)
            .await?;

        Ok(Todo::new(result.id, title, Status::Pending))
    }

    async fn cancel(&self, id: i32) -> Result<(), String> {
        query!(
            r#"UPDATE todos SET status=$1 WHERE id=$2"#,
            Status::Cancelled.to_string(),
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|res| info!("Updated : {}", res.rows_affected()))
    }

    async fn load_stream(& self) -> impl Stream<Item = Result<Todo, String>> {
        query_as!(
            TodoDao,
            r#"SELECT id, status, title, user_id, created_at FROM todos order by id"#
        )
        .fetch(&self.pool)
        .map(|res| res.map(Todo::from))
        .map_err(|e| e.to_string())
    }
}

impl TodoAdapter {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
    pub async fn load(pool: &Pool<Postgres>) -> Vec<Todo> {
        query_as!(
            TodoDao,
            r#"SELECT id, status, title, user_id, created_at FROM todos"#
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(Todo::from)
        .collect()
    }
}

impl From<TodoDao> for Todo {
    fn from(todo_model: TodoDao) -> Self {
        let s = todo_model.status.as_str();
        let status = Status::from_str(s).unwrap();
        Todo::new(todo_model.id, todo_model.title.clone(), status)
    }
}

// UserDao logic
#[derive(FromRow)]
pub struct UserDao {
    pub id: i32,
    pub login: String,
    pub password: String,
}

impl UserDao {
    pub async fn fetch(pool: &Pool<Postgres>, login: &String) -> Option<User> {
        query_as!(
            UserDao,
            r#"SELECT id, login, password FROM users WHERE login = $1"#,
            login
        )
        .fetch_one(pool)
        .await
        .map(|u| User::new(u.id, u.login, u.password))
        .ok()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::thread;

    #[derive(Debug)]
    struct Count {
        count: u32,
    }
    #[test]
    fn test() {
        let counter = Arc::new(Mutex::new(Count { count: 0 }));
        let mut handles = vec![];
        for _ in 0..100 {
            let counter_cloned = counter.clone();
            let handle = thread::spawn(move || {
                let mut guard = counter_cloned.lock().unwrap();
                guard.count = guard.count + 1;
                println!("Hello, world! {:?}", guard.count);
            });
            handles.push(handle);
        }
        handles.into_iter().for_each(|h| h.join().unwrap());
    }
}
