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
        last_commit_hash -> Nullable<Bytea>,
        data_hash -> Nullable<Bytea>,
        last_results_hash -> Nullable<Bytea>,
        evidence_hash -> Nullable<Bytea>,
        data -> Nullable<Array<Nullable<Bytea>>>,
    }
}
