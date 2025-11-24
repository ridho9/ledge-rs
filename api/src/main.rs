use std::env;

use anyhow::Result;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::get,
};
use ledger_messages::{
    ReadBuf, message_header_codec::MessageHeaderDecoder,
    transaction_posted_codec::TransactionPostedDecoder,
};
use rdkafka::{
    ClientConfig, Message,
    consumer::{Consumer, StreamConsumer},
};
use serde::Serialize;
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

    tokio::spawn(consume(pool.clone()));

    let app = Router::new()
        .route("/", get(|| async { "hello world" }))
        .route("/transaction/{id}", get(get_transaction))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    println!("Running on 0.0.0.0:{}", port);
    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(sqlx::FromRow, Serialize)]
struct DbTransaction {
    transaction_id: i64,
    account_id: i64,
    amount: i64,
    timestamp: i64,
}

async fn get_transaction(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<DbTransaction>, (StatusCode, String)> {
    let query = sqlx::query_as!(
        DbTransaction,
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

async fn consume(pool: PgPool) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", "ledger_group")
        .set("bootstrap.servers", "localhost:9092")
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("failed consumer create");

    consumer
        .subscribe(&["transactions"])
        .expect("consumer failed subscribe");

    loop {
        match consumer.recv().await {
            Ok(msg) => {
                if let Some(payload) = msg.payload() {
                    handle_payload(&pool, payload).await;
                }
            }
            Err(err) => eprintln!("consume recv error: {}", err),
        }
    }
}

async fn handle_payload(pool: &PgPool, msg: &[u8]) {
    let buf = ReadBuf::new(msg);
    let header = MessageHeaderDecoder::default().wrap(buf, 0);
    let mut transaction_posted = TransactionPostedDecoder::default();

    transaction_posted = transaction_posted.header(header, 0);
    let tx_id = transaction_posted.transaction_id();
    let account_id = transaction_posted.account_id();
    let amount = transaction_posted.amount();
    let timestamp = transaction_posted.timestamp();

    let result = sqlx::query!(
        "INSERT INTO transactions (transaction_id, account_id, amount, timestamp) 
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (transaction_id) DO NOTHING",
        tx_id as i64,
        account_id as i64,
        amount,
        timestamp as i64
    )
    .execute(pool)
    .await;

    match result {
        Ok(result) => {
            if result.rows_affected() == 0 {
                println!("tx {} already inserted, nothing inserted", tx_id)
            } else {
                println!("inserted tx {}", tx_id)
            }
        }
        Err(err) => eprintln!("error inserting tx {}: {}", tx_id, err),
    }
}
