// @generated automatically by Diesel CLI.

diesel::table! {
    block (height) {
        height -> Int8,
        block_hash -> Bytea,
        chain_id -> Text,
        time -> Timestamptz,
        app_hash -> Bytea,
        validators_hash -> Bytea,
        next_validators_hash -> Bytea,
        consensus_hash -> Bytea,
        proposer -> Bytea,
        gas_used -> Int8,
        last_commit_hash -> Nullable<Bytea>,
        data_hash -> Nullable<Bytea>,
        last_results_hash -> Nullable<Bytea>,
        evidence_hash -> Nullable<Bytea>,
    }
}

diesel::table! {
    fee (block_height, tx_idx_in_block, fee_idx_in_tx) {
        block_height -> Int8,
        tx_idx_in_block -> Int8,
        fee_idx_in_tx -> Int8,
        amount -> Numeric,
        denom -> Text,
    }
}

diesel::table! {
    msg (block_height, tx_idx_in_block, msg_idx_in_tx) {
        block_height -> Int8,
        tx_idx_in_block -> Int8,
        msg_idx_in_tx -> Int8,
        type_url -> Text,
        value -> Bytea,
    }
}

diesel::table! {
    signature (block_height, tx_idx_in_block, signature_idx_in_tx) {
        block_height -> Int8,
        tx_idx_in_block -> Int8,
        signature_idx_in_tx -> Int8,
        bz -> Bytea,
    }
}

diesel::table! {
    tx (block_height, tx_idx_in_block) {
        block_height -> Int8,
        tx_idx_in_block -> Int8,
        tx_hash -> Bytea,
        memo -> Nullable<Text>,
        timeout_height -> Nullable<Int8>,
        signers -> Jsonb,
        payer -> Bytea,
        granter -> Nullable<Bytea>,
        gas_limit -> Int8,
        gas_wanted -> Int8,
        gas_used -> Int8,
        code -> Int4,
        codespace -> Nullable<Text>,
        data_bz -> Nullable<Bytea>,
        tx_bz -> Bytea,
    }
}

diesel::joinable!(fee -> block (block_height));
diesel::joinable!(msg -> block (block_height));
diesel::joinable!(signature -> block (block_height));
diesel::joinable!(tx -> block (block_height));

diesel::allow_tables_to_appear_in_same_query!(
    block,
    fee,
    msg,
    signature,
    tx,
);
