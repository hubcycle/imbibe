-- Your SQL goes here
--

CREATE TABLE IF NOT EXISTS block (
    -- Primary key: block height
    height BIGINT PRIMARY KEY,

    block_hash BYTEA NOT NULL UNIQUE,

    -- Header fields
    chain_id TEXT NOT NULL,
    time TIMESTAMPTZ NOT NULL,
    app_hash BYTEA NOT NULL,
    validators_hash BYTEA NOT NULL,
    next_validators_hash BYTEA NOT NULL,
    consensus_hash BYTEA NOT NULL,
    proposer_address BYTEA NOT NULL,

    gas_used BIGINT NOT NULL,
    last_commit_hash BYTEA,
    data_hash BYTEA,
    last_results_hash BYTEA,
    evidence_hash BYTEA,

    -- Constraints for SHA256 (32 bytes) and address (20 bytes)
    CONSTRAINT chk_block_hash_len CHECK (
        block_hash IS NULL OR OCTET_LENGTH(block_hash) = 32
    ),
    CONSTRAINT chk_validators_hash_len CHECK (
        OCTET_LENGTH(validators_hash) = 32
    ),
    CONSTRAINT chk_next_validators_hash_len CHECK (
        OCTET_LENGTH(next_validators_hash) = 32
    ),
    CONSTRAINT chk_consensus_hash_len CHECK (
        OCTET_LENGTH(consensus_hash) = 32
    ),
    CONSTRAINT chk_proposer_address_len CHECK (
        OCTET_LENGTH(proposer_address) = 20
    ),
    CONSTRAINT chk_last_commit_hash_len CHECK (
        last_commit_hash IS NULL OR OCTET_LENGTH(last_commit_hash) = 32
    ),
    CONSTRAINT chk_data_hash_len CHECK (
        data_hash IS NULL OR OCTET_LENGTH(data_hash) = 32
    ),
    CONSTRAINT chk_last_results_hash_len CHECK (
        last_results_hash IS NULL OR OCTET_LENGTH(last_results_hash) = 32
    ),
    CONSTRAINT chk_evidence_hash_len CHECK (
        evidence_hash IS NULL OR OCTET_LENGTH(evidence_hash) = 32
    )
);
