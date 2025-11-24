# ledge-rs: Trading Ledger in Rust

**ledge-rs** is a proof-of-concept trading system built in Rust, 
designed to demostrate a **low-latency "Hot Path"** decoupled from **low latency "Cold Path"**.

## Architecture

This system uses two-tier architecture:

1. **Hot Path (Engine)**: Designed for minimal latency. The goal is to keep processing within a single-digit microsecond budget. Uses ZeroMQ for transport and SBE for zero-copy serialization.
   It processess command and offloads durability events asynchronously.
2. **Cold Path (API)**: Optimized for throughput and query. Consumes events from Kafka, stores in Postgres, and serves data via Rest API.

## Stack
1. Language: Rust
2. Transport: ZeroMQ
3. Serialization: SBE
4. Event Streaming: `rdkafka`
5. Runtime: Tokio
6. Web Framework: Axum
7. Persistence: sqlx

## Getting Started

### Prerequisites
1. Rust and Cargo
2. Docker & Docker Compose, run `docker compose up -d`
3. Java

### Database Setup
```
sqlx migrate run
```

### Running the components
You will need 3 terminals.

**Terminal A: Engine**
```
cargo run -p engine
```

**Terminal B: API**
```
cargo run -p api
```

**Terminal C: Client**
```bash
# Syntax: --acc <ID> --tx <ID> --amt <AMOUNT>
cargo run -p client -- --acc 50 --tx 1001 --amt 999
```

### 4. Verification
After sending a transaction via the client:
1.  **Engine** logs receiving the command and dispatching to Kafka.
2.  **API** logs consuming the event and inserting into Postgres.
3.  **Verify via HTTP:**
    ```bash
    curl http://localhost:5000/transaction/1001
    ```

### Cleanup
```
docker compose down
```

## Design Decision

### 1. SBE (Simple Binary Encoding) vs JSON/Profobuf.
We choose SBE to minimize garbage collection and heap allocation.
* Zero Copy: the `sbe-all.jar` generates a Rust Encoder and Decoder, that wraps the bytes buffer with a flyweight pattern,
  rather than copying it to a new struct.
* Fixed Offset: Field access is O(1) pointer arithmetic operation.

### Non-Blocking Event Production
The Engine must reply to the Client immediately, and it can't wait for Kafka produce acknowledgement.
* To solve this we produce the Kafka message using `tokio::spawn` background task.
* We also uses a stack-allocated array to copy the SBE message, allowing data to move across thread boundary and minimizing allocation using `vec!`.

### Event Sourcing
The Engine did not write to the Database. It emmits an event (`TransactionPosted`) to Kafka.
* With this the engine is never blocked by I/O bound event, like database calls, disk writes.
* Using Kafka as immutable log allows us to replay all of the events in the case that the DB is dropped.

## Benchmark

TBD