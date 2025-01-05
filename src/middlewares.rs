use crate::model::User;
use crate::repository::UserDao;
use crate::{AppState, USER};
use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use sqlx::{Pool, Postgres};

pub async fn auth(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    if let Some(current_user) = authorize_current_user(&state.pool, auth_header).await {
        // State is setup here in the middleware
        Ok(USER.scope(current_user, next.run(req)).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
async fn authorize_current_user(pool: &Pool<Postgres>, auth_token: &str) -> Option<User> {
    let vec = auth_token.split("Basic ").collect::<Vec<_>>();
    println!("{}", auth_token);
    println!("{}", vec.len());
    let token = vec.get(1).unwrap();
    println!("{}", token);
    if let Ok(decoded) = BASE64_STANDARD.decode(token) {
        let result = String::from_utf8(decoded).unwrap();
        let tokens = result
            .split(':')
            .map(|user| user.to_string())
            .collect::<Vec<String>>();
        let login = tokens.first().unwrap().clone();
        let password = tokens.get(1).unwrap().clone();
        println!("login: {}", login);
        println!("password: {}", password);
        return UserDao::fetch(pool, &login)
            .await
            .filter(|u| u.password == password);
    }

    println!("Ops {}", auth_token);
    None
}
