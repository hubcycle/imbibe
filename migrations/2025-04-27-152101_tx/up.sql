-- Your SQL goes here

CREATE TABLE IF NOT EXISTS tx (
    id BIGSERIAL PRIMARY KEY,

    tx_hash BYTEA NOT NULL UNIQUE,
    block_height BIGINT NOT NULL REFERENCES block(height),

    memo TEXT,
    timeout_height BIGINT,

    signatures BYTEA[] NOT NULL,
    signers JSONB NOT NULL,
    payer TEXT NOT NULL,
    granter TEXT,
    gas_limit BIGINT NOT NULL,
    gas_wanted BIGINT NOT NULL,
    gas_used BIGINT NOT NULL,

    code INTEGER NOT NULL,
    codespace TEXT,

    data_bz BYTEA,
    tx_bz BYTEA NOT NULL,

    CONSTRAINT chk_tx_hash_len CHECK (
        OCTET_LENGTH(tx_hash) = 32
    ),
    CONSTRAINT chk_data_bz_not_empty CHECK (
        data_bz IS NULL OR OCTET_LENGTH(data_bz) > 0
    ),
    CONSTRAINT chk_tx_bz_not_empty CHECK (
        OCTET_LENGTH(tx_bz) > 0
    )
);

CREATE TABLE IF NOT EXISTS tx_fee (
    id BIGSERIAL PRIMARY KEY,

    tx_id BIGINT NOT NULL REFERENCES tx(id),

    amount BIGINT NOT NULL,
    denom TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS msg (
    id BIGSERIAL PRIMARY KEY,

    tx_id BIGINT NOT NULL REFERENCES tx(id),

    type_url TEXT NOT NULL,
    value BYTEA NOT NULL
);
