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
        proposer_address -> Bytea,
        gas_used -> Int8,
        last_commit_hash -> Nullable<Bytea>,
        data_hash -> Nullable<Bytea>,
        last_results_hash -> Nullable<Bytea>,
        evidence_hash -> Nullable<Bytea>,
    }
}

diesel::table! {
    msg (id) {
        id -> Int8,
        tx_id -> Int8,
        type_url -> Text,
        value -> Bytea,
    }
}

diesel::table! {
    tx (id) {
        id -> Int8,
        tx_hash -> Bytea,
        block_height -> Int8,
        memo -> Nullable<Text>,
        timeout_height -> Nullable<Int8>,
        signatures -> Array<Nullable<Bytea>>,
        signers -> Jsonb,
        payer -> Text,
        granter -> Nullable<Text>,
        gas_limit -> Int8,
        gas_wanted -> Int8,
        gas_used -> Int8,
        code -> Int4,
        codespace -> Nullable<Text>,
        data_bz -> Nullable<Bytea>,
        tx_bz -> Bytea,
    }
}

diesel::table! {
    tx_fee (id) {
        id -> Int8,
        tx_id -> Int8,
        amount -> Int8,
        denom -> Text,
    }
}

diesel::joinable!(msg -> tx (tx_id));
diesel::joinable!(tx -> block (block_height));
diesel::joinable!(tx_fee -> tx (tx_id));

diesel::allow_tables_to_appear_in_same_query!(
    block,
    msg,
    tx,
    tx_fee,
);
