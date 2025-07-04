-- =============================================
-- SUMMARY TABLES
-- =============================================

CREATE TABLE IF NOT EXISTS tx_summary (
    since_blocks_ago BIGINT PRIMARY KEY,
    start_block_height BIGINT NOT NULL,
    total_gas_used BIGINT NOT NULL,
    total_txs BIGINT NOT NULL,
    total_msgs BIGINT NOT NULL,
    total_signatures BIGINT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fee_summary (
    since_blocks_ago BIGINT NOT NULL,
    start_block_height BIGINT NOT NULL,
    denom TEXT NOT NULL,
    total_amount NUMERIC(39, 0) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    PRIMARY KEY (since_blocks_ago, denom),

    CONSTRAINT chk_total_amount_is_valid_u128 CHECK (
        total_amount >= 0
        AND
        total_amount <= 340282366920938463463374607431768211455 -- u128 max value
    )
);

-- =============================================
-- HELPER FUNCTIONS
-- =============================================

-- Function to get block statistics for a specific block height
CREATE OR REPLACE FUNCTION get_block_stats(target_height BIGINT)
RETURNS TABLE(
    gas_used BIGINT,
    tx_count BIGINT,
    msg_count BIGINT,
    signature_count BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COALESCE(SUM(t.gas_used), 0)::BIGINT AS gas_used,
        COUNT(t.*)::BIGINT AS tx_count,
        COALESCE(SUM(msg_counts.msg_count), 0)::BIGINT AS msg_count,
        COALESCE(SUM(sig_counts.signature_count), 0)::BIGINT AS signature_count
    FROM tx t
    LEFT JOIN (
        SELECT block_height, tx_idx_in_block, COUNT(*) AS msg_count
        FROM msg 
        WHERE block_height = target_height
        GROUP BY block_height, tx_idx_in_block
    ) msg_counts ON t.block_height = msg_counts.block_height 
                AND t.tx_idx_in_block = msg_counts.tx_idx_in_block
    LEFT JOIN (
        SELECT block_height, tx_idx_in_block, COUNT(*) AS signature_count
        FROM signature 
        WHERE block_height = target_height
        GROUP BY block_height, tx_idx_in_block
    ) sig_counts ON t.block_height = sig_counts.block_height 
                AND t.tx_idx_in_block = sig_counts.tx_idx_in_block
    WHERE t.block_height = target_height;
END;
$$ LANGUAGE plpgsql;

-- Function to get fee statistics for a specific block height
CREATE OR REPLACE FUNCTION get_block_fee_stats(target_height BIGINT)
RETURNS TABLE(
    denom TEXT,
    total_amount NUMERIC(39, 0)
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        f.denom,
        SUM(f.amount) AS total_amount
    FROM fee f
    JOIN tx t ON f.block_height = t.block_height 
                AND f.tx_idx_in_block = t.tx_idx_in_block
    WHERE f.block_height = target_height
    GROUP BY f.denom;
END;
$$ LANGUAGE plpgsql;

-- =============================================
-- SUMMARY UPDATE FUNCTIONS
-- =============================================

-- Enhanced function to update summaries starting from a specific block
CREATE OR REPLACE FUNCTION update_summaries_from_block(
    window_sizes BIGINT[], 
    current_block_height BIGINT,
    start_block BIGINT DEFAULT 0
)
RETURNS VOID AS $$
DECLARE
    window_size BIGINT;
    blocks_since_start BIGINT;
    old_block_height BIGINT;
    new_stats RECORD;
    old_stats RECORD;
    fee_stat RECORD;
    old_fee_stat RECORD;
BEGIN
    -- Skip if current block is before the start block
    IF current_block_height < start_block THEN
        RETURN;
    END IF;

    -- Calculate how many blocks have been processed since the start block
    blocks_since_start := current_block_height - start_block + 1;
    
    -- Process each window size
    FOREACH window_size IN ARRAY window_sizes
    LOOP
        -- Get stats for the new block
        SELECT * INTO new_stats FROM get_block_stats(current_block_height);
        
        IF blocks_since_start <= window_size THEN
            -- Case 1: Less than window_size blocks since start_block, just add new data
            INSERT INTO tx_summary (
                    since_blocks_ago,
                    start_block_height,
                    total_gas_used,
                    total_txs,
                    total_msgs,
                    total_signatures,
                    updated_at)
            VALUES (window_size,
                    start_block,
                    new_stats.gas_used,
                    new_stats.tx_count,
                    new_stats.msg_count,
                    new_stats.signature_count,
                    NOW())
            ON CONFLICT (since_blocks_ago) 
            DO UPDATE SET
                start_block_height = start_block,
                total_gas_used = tx_summary.total_gas_used + EXCLUDED.total_gas_used,
                total_txs = tx_summary.total_txs + EXCLUDED.total_txs,
                total_msgs = tx_summary.total_msgs + EXCLUDED.total_msgs,
                total_signatures = tx_summary.total_signatures + EXCLUDED.total_signatures,
                updated_at = NOW();
            
            -- Handle fees for new block
            FOR fee_stat IN SELECT * FROM get_block_fee_stats(current_block_height)
            LOOP
                INSERT INTO fee_summary (
                        since_blocks_ago,
                        start_block_height,
                        denom,
                        total_amount,
                        updated_at)
                VALUES (window_size, start_block, fee_stat.denom, fee_stat.total_amount, NOW())
                ON CONFLICT (since_blocks_ago, denom)
                DO UPDATE SET
                    start_block_height = start_block,
                    total_amount = fee_summary.total_amount + EXCLUDED.total_amount,
                    updated_at = NOW();
            END LOOP;
            
        ELSE
            -- Case 2: Window is full since start_block, use sliding window
            old_block_height := current_block_height - window_size;

            -- Ensure we don't go before the start block
            IF old_block_height < start_block THEN
                old_block_height := start_block;
            END IF;
            
            -- Get stats for the old block to subtract
            SELECT * INTO old_stats FROM get_block_stats(old_block_height);
            
            -- Update tx_summary
            INSERT INTO tx_summary (
                    since_blocks_ago,
                    start_block_height,
                    total_gas_used,
                    total_txs,
                    total_msgs,
                    total_signatures,
                    updated_at)
            VALUES (window_size,
                    old_block_height + 1,
                    new_stats.gas_used - old_stats.gas_used, 
                    new_stats.tx_count - old_stats.tx_count, 
                    new_stats.msg_count - old_stats.msg_count, 
                    new_stats.signature_count - old_stats.signature_count,
                    NOW())
            ON CONFLICT (since_blocks_ago) 
            DO UPDATE SET
                start_block_height = old_block_height + 1,
                total_gas_used = tx_summary.total_gas_used + EXCLUDED.total_gas_used,
                total_txs = tx_summary.total_txs + EXCLUDED.total_txs,
                total_msgs = tx_summary.total_msgs + EXCLUDED.total_msgs,
                total_signatures = tx_summary.total_signatures + EXCLUDED.total_signatures,
                updated_at = NOW();
            
            -- Handle fees: add new fees
            FOR fee_stat IN SELECT * FROM get_block_fee_stats(current_block_height)
            LOOP
                INSERT INTO fee_summary (
                        since_blocks_ago,
                        start_block_height,
                        denom,
                        total_amount,
                        updated_at)
                VALUES (window_size,
                        old_block_height + 1,
                        fee_stat.denom,
                        fee_stat.total_amount,
                        NOW())
                ON CONFLICT (since_blocks_ago, denom)
                DO UPDATE SET
                    start_block_height = old_block_height + 1,
                    total_amount = fee_summary.total_amount + EXCLUDED.total_amount,
                    updated_at = NOW();
            END LOOP;
            
            -- Handle fees: subtract old fees (only if old_block_height > start_block_height)
            IF old_block_height > start_block THEN
                FOR old_fee_stat IN SELECT * FROM get_block_fee_stats(old_block_height)
                LOOP
                    UPDATE fee_summary 
                    SET total_amount = total_amount - old_fee_stat.total_amount,
                        updated_at = NOW()
                    WHERE since_blocks_ago = window_size 
                      AND denom = old_fee_stat.denom;
                    
                    -- Remove entries that become zero
                    DELETE FROM fee_summary 
                    WHERE since_blocks_ago = window_size 
                      AND denom = old_fee_stat.denom 
                      AND total_amount = 0;
                END LOOP;
            END IF;
        END IF;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Function to initialize summaries from a specific starting block
CREATE OR REPLACE FUNCTION initialize_summaries_from_block(
    window_sizes BIGINT[],
    start_block BIGINT,
    end_block_height BIGINT DEFAULT NULL
)
RETURNS VOID AS $$
DECLARE
    window_size BIGINT;
    current_height BIGINT;
    max_height BIGINT;
BEGIN
    -- Get the maximum block height if end_block_height is not specified
    IF end_block_height IS NULL THEN
        SELECT MAX(height) INTO max_height FROM block WHERE height >= start_block;
    ELSE
        max_height := end_block_height;
    END IF;
    
    -- Return if no blocks found
    IF max_height IS NULL THEN
        RETURN;
    END IF;
    
    -- Clear existing summaries for these window sizes and start block
    FOREACH window_size IN ARRAY window_sizes
    LOOP
        DELETE FROM tx_summary 
        WHERE since_blocks_ago = window_size AND start_block_height = start_block;
        
        DELETE FROM fee_summary 
        WHERE since_blocks_ago = window_size AND start_block_height = start_block;
    END LOOP;
    
    -- Process each block from start to end
    FOR current_height IN start_block..max_height
    LOOP
        PERFORM update_summaries_from_block(window_sizes, current_height, start_block);
    END LOOP;
END;
$$ LANGUAGE plpgsql;


-- =============================================
-- EXAMPLE USAGE QUERIES
-- =============================================

/*
-- Example: Initialize summaries for existing blocks starting from block 500
SELECT initialize_summaries_from_block(ARRAY[100, 1000, 10000], 500);

-- Example: Update summaries for a new block (called from your Rust code)
SELECT update_summaries_from_block(ARRAY[100, 1000, 10000], 1500, 500);

-- Example: Get all summary statistics
SELECT * FROM get_summary_stats();

-- Example: Get summary stats for specific window size
SELECT * FROM get_summary_stats(1000, NULL);

-- Example: Get summary stats for specific window and start block
SELECT * FROM get_summary_stats(1000, 500);

-- Example: Get fee summary statistics
SELECT * FROM get_fee_summary_stats();

-- Example: Get fee summary for specific denom
SELECT * FROM get_fee_summary_stats(NULL, NULL, 'uatom');

-- Example: Get complete window summary (tx + fees in one query)
SELECT * FROM get_window_summary(1000, 500);

-- Example: Clean up summary data for specific configuration
SELECT cleanup_summary_data(ARRAY[100, 1000], 500);
*/
