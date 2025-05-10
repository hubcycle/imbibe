use core::num::NonZeroU64;

use diesel::{prelude::QueryableByName, sql_types::BigInt};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use futures::{Stream, StreamExt, TryStreamExt};

use crate::domain::{block::Block, tx::TxResult};

use super::{
	DbConn,
	record::insert::{NewBlockRecord, NewMsgRecord, NewTxFeeRecord, NewTxRecord},
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
	let mut new_block_records = vec![];
	let mut new_tx_records = vec![];

	let mut new_fee_builders = vec![];
	let mut new_msg_builders = vec![];

	for (block, txrs) in blocks_with_tx_results {
		new_block_records.push(NewBlockRecord::try_from(block)?);

		for txr in txrs.as_ref().iter() {
			let new_tx_record = NewTxRecord::try_from(txr)?;
			new_tx_records.push(new_tx_record);

			for coin in txr.fee() {
				let fee_sans_tx_id_record = NewTxFeeRecord::builder()
					.amount(coin.amount.try_into()?)
					.denom(coin.denom.as_ref());

				new_fee_builders.push(fee_sans_tx_id_record);
			}

			let msg_sans_tx_id_records = txr
				.msgs()
				.iter()
				.map(|m| NewMsgRecord::builder().type_url(&m.type_url).value(&m.value));

			new_msg_builders.extend(msg_sans_tx_id_records);
		}
	}

	conn.transaction(|conn| {
		async move {
			diesel::insert_into(schema::block::table)
				.values(new_block_records)
				.execute(conn)
				.await?;

			let new_tx_ids: Vec<i64> = diesel::insert_into(schema::tx::table)
				.values(new_tx_records)
				.returning(schema::tx::id)
				.get_results(conn)
				.await?;

			let (new_fee_records, new_msg_records): (Vec<_>, Vec<_>) = new_tx_ids
				.into_iter()
				.zip(new_fee_builders.into_iter().zip(new_msg_builders))
				.map(|(id, (frb, mrb))| (frb.tx_id(id).build(), mrb.tx_id(id).build()))
				.unzip();

			diesel::insert_into(schema::tx_fee::table)
				.values(new_fee_records)
				.execute(conn)
				.await?;

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

	let new_tx_records: Vec<_> =
		tx_results.iter().map(NewTxRecord::try_from).collect::<anyhow::Result<_>>()?;

	conn.transaction(|conn| {
		async move {
			diesel::insert_into(schema::block::table)
				.values(new_block_record)
				.execute(conn)
				.await?;

			let new_tx_ids: Vec<i64> = diesel::insert_into(schema::tx::table)
				.values(new_tx_records)
				.returning(schema::tx::id)
				.get_results(conn)
				.await?;

			let mut new_fee_records = vec![];
			let mut new_msg_records = vec![];

			for (txr, tx_id) in tx_results.iter().zip(new_tx_ids) {
				for coin in txr.fee() {
					let tx_fee_record = NewTxFeeRecord::builder()
						.tx_id(tx_id)
						.amount(coin.amount.try_into()?)
						.denom(coin.denom.as_ref())
						.build();

					new_fee_records.push(tx_fee_record);
				}

				let msg_records = txr.msgs().iter().map(|m| {
					NewMsgRecord::builder()
						.tx_id(tx_id)
						.type_url(&m.type_url)
						.value(&m.value)
						.build()
				});

				new_msg_records.extend(msg_records);
			}

			diesel::insert_into(schema::tx_fee::table)
				.values(new_fee_records)
				.execute(conn)
				.await?;

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
