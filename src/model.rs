use diesel::{PgConnection, Queryable, Selectable};
use crate::schema::todos::dsl::todos;
use crate::schema::todos::title;
use diesel::prelude::*;
use serde::Serialize;
/*impl Todo12 {
    pub fn new(title: String, status: Status) -> Self {

    }
}

pub struct Todo12 {
    title: String,
    status: Status,
}*/

#[derive(Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::schema::todos)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct TodosToPersist {
    pub id: i32,
    pub title: String,
    status: String,
}

impl TodosToPersist {
    pub fn load(connection: &mut PgConnection) -> Vec<Self> {

        let results = todos
            .limit(5)
            .select(TodosToPersist::as_select())
            .load(connection)
            .expect("Error loading posts .");
        results
    }
}


pub enum Status {
    Active,
    Pending,
    Cancelled,
}
