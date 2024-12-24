// @generated automatically by Diesel CLI.

diesel::table! {
    todos (id) {
        id -> Int4,
        #[max_length = 256]
        title -> Varchar,
        #[max_length = 256]
        status -> Varchar,
    }
}
