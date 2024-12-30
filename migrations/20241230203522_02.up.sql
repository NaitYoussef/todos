-- Add up migration script here
CREATE TABLE users
(
    id     SERIAL PRIMARY KEY,
    login  VARCHAR(256) NOT NULL,
    password VARCHAR(256) NOT NULL
)