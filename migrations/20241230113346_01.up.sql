-- Your SQL goes here
CREATE TABLE todos
(
    id     SERIAL PRIMARY KEY,
    title  VARCHAR(256) NOT NULL,
    status VARCHAR(256) NOT NULL
)