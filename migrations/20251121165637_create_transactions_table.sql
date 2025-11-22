CREATE TABLE
    IF NOT EXISTS transactions (
        transaction_id BIGINT PRIMARY KEY,
        account_id BIGINT NOT NULL,
        amount BIGINT NOT NULL,
        timestamp BIGINT NOT NULL
    );