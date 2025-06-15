CREATE TABLE IF NOT EXISTS tx (
    block_height BIGINT NOT NULL REFERENCES block(height),
    tx_idx_in_block BIGINT NOT NULL,

    tx_hash BYTEA NOT NULL UNIQUE,

    memo TEXT,
    timeout_height BIGINT,

    signers JSONB NOT NULL,

    payer BYTEA NOT NULL,
    granter BYTEA,

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
    CONSTRAINT chk_memo_not_empty CHECK (
        memo IS NULL OR LENGTH(memo) > 0
    ),
    CONSTRAINT chk_timeout_height_positive CHECK (
      timeout_height IS NULL OR timeout_height > 0  
    ),
    CONSTRAINT chk_payer_len CHECK (
        OCTET_LENGTH(payer) = 20
    ),
    CONSTRAINT chk_granter_len CHECK (
        granter IS NULL OR OCTET_LENGTH(granter) = 20
    ),
    CONSTRAINT chk_codespace_not_empty CHECK (
        codespace IS NULL OR LENGTH(codespace) > 0
    ),
    CONSTRAINT chk_data_bz_not_empty CHECK (
        data_bz IS NULL OR OCTET_LENGTH(data_bz) > 0
    ),
    CONSTRAINT chk_tx_bz_not_empty CHECK (
        OCTET_LENGTH(tx_bz) > 0
    )
);

CREATE TABLE IF NOT EXISTS signature (
    block_height BIGINT NOT NULL REFERENCES block(height),
    tx_idx_in_block BIGINT NOT NULL,
    signature_idx_in_tx BIGINT NOT NULL,

    bz BYTEA NOT NULL,

    PRIMARY KEY (block_height, tx_idx_in_block, signature_idx_in_tx),
    FOREIGN KEY (block_height, tx_idx_in_block) REFERENCES tx(block_height, tx_idx_in_block),

    CONSTRAINT chk_bz_not_empty CHECK (
        OCTET_LENGTH(bz) > 0
    )        
);

CREATE TABLE IF NOT EXISTS fee (
    block_height BIGINT NOT NULL REFERENCES block(height),
    tx_idx_in_block BIGINT NOT NULL,
    fee_idx_in_tx BIGINT NOT NULL,

    amount NUMERIC(39, 0) NOT NULL,
    denom TEXT NOT NULL,

    PRIMARY KEY (block_height, tx_idx_in_block, fee_idx_in_tx),
    FOREIGN KEY (block_height, tx_idx_in_block) REFERENCES tx(block_height, tx_idx_in_block),

    CONSTRAINT chk_amount_is_valid_u128 CHECK (
        amount >= 0
        AND
        amount <= 340282366920938463463374607431768211455 -- u128 max value
    )
);

CREATE TABLE IF NOT EXISTS msg (
    block_height BIGINT NOT NULL REFERENCES block(height),
    tx_idx_in_block BIGINT NOT NULL,
    msg_idx_in_tx BIGINT NOT NULL,

    type_url TEXT NOT NULL,
    value BYTEA NOT NULL,

    PRIMARY KEY (block_height, tx_idx_in_block, msg_idx_in_tx),
    FOREIGN KEY (block_height, tx_idx_in_block) REFERENCES tx(block_height, tx_idx_in_block)
);
