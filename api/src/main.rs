use std::env;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;
    let port = env::var("PORT")?;
    let db_url = env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let app = Router::new()
        .route("/", get(|| async { "hello world" }))
        .route("/transaction/{id}", get(get_transaction))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Running on 0.0.0.0:{}", port);
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct Transaction {
    transaction_id: i64,
    account_id: i64,
    amount: i64,
    timestamp: i64,
}

async fn get_transaction(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<Transaction>, (StatusCode, String)> {
    let query = sqlx::query_as!(
        Transaction,
        "
    select * from transactions
    where transaction_id = $1;
    ",
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|err| {
        eprintln!("DB Error: {}", err);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database error".to_string(),
        )
    })?
    .ok_or((
        StatusCode::NOT_FOUND,
        format!("Transaction {} not found", id),
    ))?;

    Ok(Json(query))
}
