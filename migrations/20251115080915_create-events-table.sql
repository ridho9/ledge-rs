-- Add migration script here
CREATE EXTENSION IF NOT EXISTS pg_uuidv7;

CREATE TABLE
    events (
        id UUID PRIMARY KEY DEFAULT uuid_generate_v7 (),
        aggregate_id UUID NOT NULL,
        sequence BIGINT NOT NULL,
        event_type TEXT NOT NULL,
        payload JSONB NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        UNIQUE (aggregate_id, sequence)
    );