use crate::model::{Todo, TodoPort};
use crate::usecase::TodoError::{AlreadyCancel, DatabaseError, NotFound};
use futures::TryFutureExt;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
pub enum TodoError {
    AlreadyCancel,
    NotFound,
    DatabaseError,
}

impl Error for TodoError {}

impl Display for TodoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AlreadyCancel => write!(f, "AlreadyCancel"),
            NotFound => write!(f, "NotFound"),
            DatabaseError => write!(f, "DatabaseError"),
        }
    }
}

pub async fn cancel_todo(port: &impl TodoPort, id: i32, _user_id: i32) -> Result<(), TodoError> {
    if let Some(mut todo) = port.load_by_id(id).await {
        if !todo.cancel() {
            return Err(AlreadyCancel);
        }
        port.cancel(todo.id()).map_err(|_err| DatabaseError).await
    } else {
        Err(NotFound)
    }
}

pub async fn create_todo(
    port: &impl TodoPort,
    title: String,
    user_id: i32,
) -> Result<Todo, Box<dyn Error>> {
    port.insert_new_todo(title, user_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{MockTodoPort, Status};
    use mockall::predicate;

    #[tokio::test]
    async fn should_cancel_when_state_exists_and_is_pending() {
        // Given
        let mut todo_port = MockTodoPort::new();
        todo_port.expect_load_by_id()
            .with(predicate::eq(1))
            .returning(|id| Some(Todo::new(id, "".to_string(), Status::Pending)));

        todo_port.expect_cancel()
            .with(predicate::eq(1))
            .returning(|_id| Ok(()));

        // When
        let todo = cancel_todo(&todo_port, 1, 1).await;

        // Then
        assert_eq!(Ok(()), todo)
    }

    #[tokio::test]
    async fn should_return_not_found_where_todo_not_exist() {
        // Given
        let mut todo_port = MockTodoPort::new();
        todo_port.expect_load_by_id()
            .with(predicate::eq(1))
            .returning(|_id| None);

        // When
        let todo = cancel_todo(&todo_port, 1, 1).await;

        // Then
        assert_eq!(Err(NotFound), todo)
    }

    #[tokio::test]
    async fn should_return_already_cancel_where_todo_is_cancel() {
        // Given
        let mut todo_port = MockTodoPort::new();
        todo_port.expect_load_by_id()
            .with(predicate::eq(1))
            .returning(|id| Some(Todo::new(id, "".to_string(), Status::Cancelled)));

        // When
        let todo = cancel_todo(&todo_port, 1, 1).await;

        // Then
        assert_eq!(Err(AlreadyCancel), todo)
    }
}
