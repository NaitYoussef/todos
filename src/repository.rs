use crate::model::{Status, Todo, User};
use crate::resource::TodoResourceV1;
use axum::body::Bytes;
use chrono::{DateTime, Utc};
use futures::{FutureExt, StreamExt, TryStreamExt};
use serde::Serialize;
use sqlx::{query, query_as, FromRow, Pool, Postgres};
use std::str::FromStr;
use tokio_stream::Stream;
use tracing::info;

// TodoDao logic
#[derive(Serialize, FromRow)]
pub struct TodoDao {
    id: i32,
    title: String,
    status: String,
    user_id: i32,
    created_at: DateTime<Utc>,
}

impl TodoDao {
    pub async fn load(pool: &Pool<Postgres>) -> Vec<Todo> {
        query_as!(
            TodoDao,
            r#"SELECT id, status, title, user_id, created_at FROM todos"#
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .iter()
        .map(|todoDao| todoDao.todo())
        .collect()
    }

    pub async fn load_by_id(pool: &Pool<Postgres>, id: i32) -> Option<Todo> {
        query_as!(
            TodoDao,
            r#"SELECT id, status, title, user_id, created_at FROM todos WHERE id=$1"#,
            id
        )
        .fetch_one(pool)
        .await
        .ok()
        .map(|todoDao| todoDao.todo())
    }

    fn todo(&self) -> Todo {
        let s = self.status.as_str();
        let status = Status::from_str(s).unwrap();
        Todo::new(self.id, self.title.clone(), status)
    }

    pub async fn insert_new_todo(
        pool: &Pool<Postgres>,
        title: String,
        user_id: i32,
    ) -> Result<(), String> {
        let _ = query_as!(
            TodoDao,
            r#"INSERT INTO todos (status, title, user_id, created_at) VALUES ($1, 'PENDING', $2, $3)"#,
            title,
            user_id,
            Utc::now()
        )
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn cancel(id: i32, pool: &Pool<Postgres>) -> Result<(), String> {
        query!(
            r#"UPDATE todos SET status=$1 WHERE id=$2"#,
            Status::Cancelled.to_string(),
            id
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())
        .map(|res| info!("Updated : {}", res.rows_affected()))
    }

    pub async fn load_stream<'a>(
        pool: &'a Pool<Postgres>,
    ) -> impl Stream<Item = Result<Todo, String>> + use<'a> {
        query_as!(
            TodoDao,
            r#"SELECT id, status, title, user_id, created_at FROM todos order by id"#
        )
        .fetch(pool)
        .map(|res| res.map(|todoDao| todoDao.todo()))
        .map_err(|e| e.to_string())
    }
}

impl From<TodoDao> for Bytes {
    fn from(value: TodoDao) -> Self {
        let mut string = serde_json::to_vec(&value).unwrap();
        let mut vec = "\n".to_string().into_bytes();
        string.append(&mut vec);
        string.into()
    }
}

pub fn convert(value: &Vec<TodoResourceV1>) -> Bytes {
    let mut result = Vec::new();
    for todo in value {
        let mut string = serde_json::to_vec(&todo).unwrap();
        result.append(&mut string);
        let mut vec = "\n".to_string().into_bytes();
        result.append(&mut vec);
    }
    result.into()
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
    use std::rc::Rc;
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
            let mut counterCloned = counter.clone();
            let handle = thread::spawn(move || {
                let mut guard = counterCloned.lock().unwrap();
                guard.count = guard.count + 1;
                println!("Hello, world! {:?}", guard.count);
            });
            handles.push(handle);
        }
        handles.into_iter().for_each(|h| h.join().unwrap());
    }

    fn m1(c: Rc<Count>) {
        println!("{:?}", Rc::strong_count(&c));
    }
}
