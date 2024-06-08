use std::env;
use std::path::PathBuf;

use anyhow::Result;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Row, Sqlite};
use tokio::net::TcpListener;

async fn auth_check(
    State(pool): State<Pool<Sqlite>>,
    headers: HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let (_, token_provided) = auth_header
        .split_once(' ')
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let rows = sqlx::query("SELECT auth_token FROM users")
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token_found = rows
        .into_iter()
        .map(|row| row.try_get(0).map_err(|_| StatusCode::UNAUTHORIZED))
        .filter_map(|token| token.ok())
        .any(|token: String| token == token_provided);

    if !token_found {
        println!("{}", token_found);
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(StatusCode::OK)
}

#[tokio::main]
async fn main() -> Result<()> {
    let database_location = env::var("DATABASE_LOCATION").unwrap_or("db.sqlite".to_string());
    let database_location = PathBuf::from(database_location);
    let connection_string = format!("sqlite://{}", database_location.to_string_lossy());

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&connection_string)
        .await?;

    let app = Router::new()
        .route("/auth", post(auth_check).get(auth_check))
        .with_state(pool);

    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
