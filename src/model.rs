use axum::body::Bytes;
use futures::{StreamExt, TryStreamExt};
use serde::Serialize;
use sqlx::{query, query_as, FromRow, Pool, Postgres};
use tokio_stream::Stream;

impl Todo {
    pub fn new(id: i32, title: String, status: String) -> Self {
        Self { id, title, status }
    }

    pub async fn load(pool: &Pool<Postgres>) -> Vec<Self> {
        let query = query!(r#"SELECT status, title, id FROM todos"#)
            .fetch_all(pool)
            .await
            .unwrap();

        query
            .into_iter()
            .map(|row| Todo::new(row.id, row.title, row.status))
            .collect()
    }

    pub async fn insert_new_todo(pool: &Pool<Postgres>, title: String) -> Result<(), String> {
        let _ = query_as!(
            Todo,
            r#"INSERT INTO todos (status, title) VALUES ($1, 'PENDING')"#,
            title
        )
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn load_stream<'a>(
        pool: &'a Pool<Postgres>,
    ) -> impl Stream<Item = Result<Todo, String>> + use<'a> {
        query_as!(Todo, r#"SELECT status, title, id FROM todos order by id"#)
            .fetch(pool)
            .map_err(|e| e.to_string())
    }
    pub async fn load_stream_static(
        pool: &'static Pool<Postgres>,
    ) -> impl Stream<Item = Result<Bytes, &'static str>> + 'static {
        let pin = query!(r#"SELECT status, title, id FROM todos"#)
            .fetch(pool)
            .map(|row| match row {
                Ok(todo) => Ok(Todo::new(todo.id, todo.title, todo.status)),
                Err(_) => Err("error)"),
            })
            .map(|row| match row {
                Ok(todo) => Ok(Bytes::from(todo)),
                Err(_) => Err("error"),
            });
        pin
    }
}

#[derive(FromRow)]
pub struct User {
    pub login: String,
    pub password: String,
}

impl User {
    pub async fn fetch(pool: &Pool<Postgres>, login: &String) -> Option<User> {
        query_as!(User, r#"SELECT login, password FROM users WHERE login = $1"#, login)
            .fetch_one(pool)
            .await
            .ok()
    }
}

#[derive(Serialize, FromRow)]
pub struct Todo {
    id: i32,
    title: String,
    status: String,
}

pub enum Status {
    Active,
    Pending,
    Cancelled,
}

impl From<Todo> for Bytes {
    fn from(value: Todo) -> Self {
        let mut string = serde_json::to_vec(&value).unwrap();
        let mut vec = "\n".to_string().into_bytes();
        string.append(&mut vec);
        string.into()
    }
}
pub fn convert(value: &Vec<Todo>) -> Bytes {
    let mut result = Vec::new();
    for todo in value {
        let mut string = serde_json::to_vec(&todo).unwrap();
        result.append(&mut string);
        let mut vec = "\n".to_string().into_bytes();
        result.append(&mut vec);
    }
    result.into()
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
