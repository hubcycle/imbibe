-- Your SQL goes here

CREATE TABLE IF NOT EXISTS tx (
    block_height BIGINT NOT NULL REFERENCES block(height),
    tx_idx_in_block BIGINT NOT NULL,

    tx_hash BYTEA NOT NULL UNIQUE,

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

    PRIMARY KEY (block_height, tx_idx_in_block),

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

CREATE TABLE IF NOT EXISTS fee (
    block_height BIGINT NOT NULL REFERENCES block(height),
    tx_idx_in_block BIGINT NOT NULL,
    fee_idx_in_tx BIGINT NOT NULL,

    amount BIGINT NOT NULL,
    denom TEXT NOT NULL,

    PRIMARY KEY (block_height, tx_idx_in_block, fee_idx_in_tx),
    FOREIGN KEY (block_height, tx_idx_in_block) REFERENCES tx (block_height, tx_idx_in_block)
);

CREATE TABLE IF NOT EXISTS msg (
    block_height BIGINT NOT NULL REFERENCES block(height),
    tx_idx_in_block BIGINT NOT NULL,
    msg_idx_in_tx BIGINT NOT NULL,

    type_url TEXT NOT NULL,
    value BYTEA NOT NULL,

    PRIMARY KEY (block_height, tx_idx_in_block, msg_idx_in_tx),
    FOREIGN KEY (block_height, tx_idx_in_block) REFERENCES tx (block_height, tx_idx_in_block)
);
