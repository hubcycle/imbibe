use core::num::NonZeroU64;

use diesel::{prelude::QueryableByName, sql_types::BigInt};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use futures::{Stream, StreamExt, TryStreamExt};

use crate::domain::{block::Block, tx::TxResult};

use super::{
	DbConn,
	record::insert::{NewBlockRecord, NewFeeRecord, NewMsgRecord, NewTxRecord},
	schema,
};

#[tracing::instrument(skip_all)]
pub async fn save_blocks_with_tx_resulsts<TXRS>(
	conn: &mut DbConn,
	blocks_with_tx_results: &[(Block, TXRS)],
) -> anyhow::Result<()>
where
	TXRS: AsRef<[TxResult]>,
{
	let mut new_block_records = Vec::with_capacity(blocks_with_tx_results.len());
	let mut new_tx_records = vec![];
	let mut new_fee_records = vec![];
	let mut new_msg_records = vec![];

	for (block, txrs) in blocks_with_tx_results {
		new_block_records.push(NewBlockRecord::try_from(block)?);

		process_new_records_from_tx_results(
			txrs.as_ref(),
			&mut new_tx_records,
			&mut new_fee_records,
			&mut new_msg_records,
		)?;
	}

	conn.transaction(|conn| {
		async move {
			diesel::insert_into(schema::block::table)
				.values(new_block_records)
				.execute(conn)
				.await?;

			diesel::insert_into(schema::tx::table).values(new_tx_records).execute(conn).await?;
			diesel::insert_into(schema::fee::table).values(new_fee_records).execute(conn).await?;
			diesel::insert_into(schema::msg::table).values(new_msg_records).execute(conn).await?;

			anyhow::Ok(())
		}
		.scope_boxed()
	})
	.await?;

	Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn save_block_with_tx_results(
	conn: &mut DbConn,
	block: &Block,
	tx_results: &[TxResult],
) -> anyhow::Result<()> {
	let new_block_record = NewBlockRecord::try_from(block)?;

	let mut new_tx_records = Vec::with_capacity(tx_results.len());
	let mut new_fee_records = vec![];
	let mut new_msg_records = vec![];

	process_new_records_from_tx_results(
		tx_results,
		&mut new_tx_records,
		&mut new_fee_records,
		&mut new_msg_records,
	)?;

	conn.transaction(|conn| {
		async move {
			diesel::insert_into(schema::block::table)
				.values(new_block_record)
				.execute(conn)
				.await?;

			diesel::insert_into(schema::tx::table).values(new_tx_records).execute(conn).await?;
			diesel::insert_into(schema::fee::table).values(new_fee_records).execute(conn).await?;
			diesel::insert_into(schema::msg::table).values(new_msg_records).execute(conn).await?;

			anyhow::Ok(())
		}
		.scope_boxed()
	})
	.await?;

	Ok(())
}

#[tracing::instrument(skip(conn))]
pub async fn fetch_missing_block_heights(
	conn: &mut DbConn,
	lo: NonZeroU64,
	hi: NonZeroU64,
) -> anyhow::Result<impl Stream<Item = anyhow::Result<NonZeroU64>>> {
	#[derive(QueryableByName)]
	struct MissingHeight {
		#[diesel(sql_type = BigInt)]
		height: i64,
	}

	let sql = r#"
		SELECT gs.height
		FROM generate_series($1, $2) AS gs(height)
		LEFT JOIN block ON block.height = gs.height
		WHERE block.height IS NULL
	"#;

	let stream = diesel::sql_query(sql)
		.bind::<BigInt, _>(i64::try_from(lo.get())?)
		.bind::<BigInt, _>(i64::try_from(hi.get())?)
		.load_stream::<MissingHeight>(conn)
		.await?
		.map_err(anyhow::Error::from)
		.map_ok(|h| h.height)
		.map_ok(|h| anyhow::Ok(u64::try_from(h)?))
		.map_ok(|h| h.map(|h| Ok(NonZeroU64::try_from(h)?)))
		.map(|h| h??);

	Ok(stream)
}

fn process_new_records_from_tx_results<'a>(
	tx_results: &'a [TxResult],
	new_tx_records: &mut Vec<NewTxRecord<'a>>,
	new_fee_records: &mut Vec<NewFeeRecord<'a>>,
	new_msg_records: &mut Vec<NewMsgRecord<'a>>,
) -> anyhow::Result<()> {
	for txr in tx_results {
		new_tx_records.push(NewTxRecord::try_from(txr)?);

		let block_height = txr.block_height().get().try_into()?;
		let tx_idx_in_block = txr.tx_idx_in_block().try_into()?;

		for (idx, coin) in txr.fee().iter().enumerate() {
			let fee_record = NewFeeRecord::builder()
				.block_height(block_height)
				.tx_idx_in_block(tx_idx_in_block)
				.fee_idx_in_tx(idx.try_into()?)
				.amount(coin.amount.try_into()?)
				.denom(coin.denom.as_ref())
				.build();

			new_fee_records.push(fee_record);
		}

		for (idx, msg) in txr.msgs().iter().enumerate() {
			let msg_record = NewMsgRecord::builder()
				.block_height(block_height)
				.tx_idx_in_block(tx_idx_in_block)
				.msg_idx_in_tx(idx.try_into()?)
				.type_url(&msg.type_url)
				.value(&msg.value)
				.build();

			new_msg_records.push(msg_record);
		}
	}

	Ok(())
}
