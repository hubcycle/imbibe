DROP FUNCTION IF EXISTS initialize_summaries_from_block(BIGINT[], BIGINT, BIGINT);
DROP FUNCTION IF EXISTS update_summaries_from_block(BIGINT[], BIGINT, BIGINT);
DROP FUNCTION IF EXISTS get_block_stats(BIGINT);
DROP FUNCTION IF EXISTS get_block_fee_stats(BIGINT);

DROP TABLE IF EXISTS tx_summary;
DROP TABLE IF EXISTS fee_summary;
