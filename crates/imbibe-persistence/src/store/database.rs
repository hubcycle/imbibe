use core::{
	num::NonZeroU64,
	ops::{Bound, RangeBounds},
};

use std::sync::LazyLock;

use diesel::{
	ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, dsl,
	expression::SqlLiteral,
	prelude::QueryableByName,
	sql_types::{Array, BigInt, Bytea},
};
use diesel_async::{AsyncConnection, RunQueryDsl, scoped_futures::ScopedFutureExt};
use futures::{Stream, StreamExt, TryStreamExt};
use imbibe_domain::{
	Sha256,
	block::Block,
	summary::{FeeSummary, TxSummary},
	tx::{Fees, Tx},
};

use crate::{
	pool::DbConn,
	record::{
		insert::{NewBlockRecord, NewFeeRecord, NewMsgRecord, NewSignatureRecord, NewTxRecord},
		select::{
			BlockWithDataRecord, FeeRecord, FeeSummaryRecord, MsgRecord, SignatureRecord, TxRecord,
			TxSummaryRecord, TxWithDetailsRecord,
		},
	},
	schema,
};

use super::{InvalidValueError, StoreError};

static TX_BZ_ARRAY_AGG: LazyLock<SqlLiteral<Array<Bytea>>> = LazyLock::new(|| {
	dsl::sql::<Array<Bytea>>(
		"COALESCE(NULLIF(array_agg(tx.tx_bz ORDER BY tx.tx_idx_in_block ASC), '{NULL}'), '{}')",
	)
});

#[tracing::instrument(skip_all)]
pub async fn save_blocks_with_txs<TXS>(
	conn: &mut DbConn,
	blocks_with_txs: &[(Block, TXS)],
) -> Result<(), StoreError>
where
	TXS: AsRef<[Tx]>,
{
	let mut new_block_records = Vec::with_capacity(blocks_with_txs.len());
	let mut new_tx_records = vec![];
	let mut new_signature_records = vec![];
	let mut new_fee_records = vec![];
	let mut new_msg_records = vec![];

	for (block, txs) in blocks_with_txs {
		new_block_records.push(NewBlockRecord::try_from(block)?);

		process_new_records_from_txs(
			txs.as_ref(),
			&mut new_tx_records,
			&mut new_signature_records,
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
			diesel::insert_into(schema::signature::table)
				.values(new_signature_records)
				.execute(conn)
				.await?;
			diesel::insert_into(schema::fee::table).values(new_fee_records).execute(conn).await?;
			diesel::insert_into(schema::msg::table).values(new_msg_records).execute(conn).await?;

			Result::<_, StoreError>::Ok(())
		}
		.scope_boxed()
	})
	.await?;

	Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn save_block_with_txs(
	conn: &mut DbConn,
	block: &Block,
	txs: &[Tx],
) -> Result<(), StoreError> {
	let new_block_record = NewBlockRecord::try_from(block)?;

	let mut new_tx_records = Vec::with_capacity(txs.len());
	let mut new_signature_records = vec![];
	let mut new_fee_records = vec![];
	let mut new_msg_records = vec![];

	process_new_records_from_txs(
		txs,
		&mut new_tx_records,
		&mut new_signature_records,
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
			diesel::insert_into(schema::signature::table)
				.values(new_signature_records)
				.execute(conn)
				.await?;
			diesel::insert_into(schema::fee::table).values(new_fee_records).execute(conn).await?;
			diesel::insert_into(schema::msg::table).values(new_msg_records).execute(conn).await?;

			Result::<_, StoreError>::Ok(())
		}
		.scope_boxed()
	})
	.await?;

	Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn fetch_missing_block_heights<HR>(
	conn: &mut DbConn,
	height_range: HR,
) -> Result<impl Stream<Item = Result<NonZeroU64, StoreError>> + use<HR>, StoreError>
where
	HR: RangeBounds<NonZeroU64>,
{
	#[derive(QueryableByName)]
	struct MissingHeight {
		#[diesel(sql_type = BigInt)]
		height: i64,
	}

	let (lo, hi) = lo_hi_from_bounds(height_range)?;

	tracing::info!("fetching missing blocks from {lo} to {hi}");

	const SQL: &str = r#"
		SELECT gs.height
		FROM generate_series($1, $2) AS gs(height)
		LEFT JOIN block ON block.height = gs.height
		WHERE block.height IS NULL
	"#;

	let stream = diesel::sql_query(SQL)
		.bind::<BigInt, _>(i64::try_from(lo.get()).map_err(InvalidValueError::from)?)
		.bind::<BigInt, _>(i64::try_from(hi.get()).map_err(InvalidValueError::from)?)
		.load_stream::<MissingHeight>(conn)
		.await?
		.map_ok(|h| h.height)
		.map_ok(u64::try_from)
		.map_ok(|h| h.and_then(TryFrom::try_from).map_err(InvalidValueError::from))
		.map_err(From::from)
		.map(|h| h.and_then(|h| h.map_err(From::from)));

	Ok(stream)
}

#[tracing::instrument(skip_all)]
pub async fn fetch_blocks_in_height_range<HR>(
	conn: &mut DbConn,
	height_range: HR,
) -> Result<impl Stream<Item = Result<Block, StoreError>> + use<HR>, StoreError>
where
	HR: RangeBounds<NonZeroU64>,
{
	let (lo, hi) = lo_hi_from_bounds(height_range)?;

	tracing::info!("fetching blocks from {lo} to {hi}");

	let stream = schema::block::table
		.left_join(schema::tx::table.on(schema::tx::block_height.eq(schema::block::height)))
		.filter(schema::block::height.ge(i64::try_from(lo.get()).map_err(InvalidValueError::from)?))
		.filter(schema::block::height.le(i64::try_from(hi.get()).map_err(InvalidValueError::from)?))
		.select((schema::block::all_columns, &*TX_BZ_ARRAY_AGG))
		.group_by(schema::block::all_columns)
		.order(schema::block::height.desc())
		.load_stream::<BlockWithDataRecord>(conn)
		.await?
		.map_ok(TryFrom::try_from)
		.map_err(From::from)
		.map(|res| res.and_then(|block| block.map_err(From::from)));

	Ok(stream)
}

#[tracing::instrument(skip(conn))]
pub async fn fetch_txs_by_height(
	conn: &mut DbConn,
	height: NonZeroU64,
) -> Result<Vec<Tx>, StoreError> {
	let mut txs = vec![];

	for idx in 0.. {
		match fetch_tx_by_block_height_and_tx_idx_in_block(conn, height, idx).await? {
			Some(tx) => txs.push(tx),
			None => break,
		}
	}

	Ok(txs)
}

#[tracing::instrument(skip_all)]
pub async fn fetch_tx_summaries(
	conn: &mut DbConn,
) -> Result<impl Stream<Item = Result<TxSummary, StoreError>> + use<>, StoreError> {
	let stream = schema::tx_summary::table
		.select(schema::tx_summary::all_columns)
		.order(schema::tx_summary::since_blocks_ago.asc())
		.load_stream::<TxSummaryRecord>(conn)
		.await?
		.map_ok(TryFrom::try_from)
		.map_ok(|summary| summary.map_err(StoreError::from))
		.map(|res| res?)
		.map_err(From::from);

	Ok(stream)
}

#[tracing::instrument(skip_all)]
pub async fn fetch_fee_summaries(
	conn: &mut DbConn,
) -> Result<impl Stream<Item = Result<FeeSummary, StoreError>> + use<>, StoreError> {
	let stream = schema::fee_summary::table
		.select(schema::fee_summary::all_columns)
		.order(schema::fee_summary::since_blocks_ago.asc())
		.load_stream::<FeeSummaryRecord>(conn)
		.await?
		.map_ok(TryFrom::try_from)
		.map_ok(|summary| summary.map_err(StoreError::from))
		.map(|res| res?)
		.map_err(From::from);

	Ok(stream)
}

#[tracing::instrument(skip(conn))]
pub async fn fetch_tx_summary_since_blocks_ago(
	conn: &mut DbConn,
	since_blocks_ago: NonZeroU64,
) -> Result<Option<TxSummary>, StoreError> {
	let since_blocks_ago =
		i64::try_from(since_blocks_ago.get()).map_err(InvalidValueError::from)?;

	schema::tx_summary::table
		.select(schema::tx_summary::all_columns)
		.filter(schema::tx_summary::since_blocks_ago.eq(since_blocks_ago))
		.first::<TxSummaryRecord>(conn)
		.await
		.optional()?
		.map(TryFrom::try_from)
		.transpose()
		.map_err(From::from)
}

#[tracing::instrument(skip(conn))]
pub async fn fetch_fee_summary_since_blocks_ago(
	conn: &mut DbConn,
	since_blocks_ago: NonZeroU64,
) -> Result<Option<FeeSummary>, StoreError> {
	let since_blocks_ago =
		i64::try_from(since_blocks_ago.get()).map_err(InvalidValueError::from)?;

	schema::fee_summary::table
		.select(schema::fee_summary::all_columns)
		.filter(schema::fee_summary::since_blocks_ago.eq(since_blocks_ago))
		.first::<FeeSummaryRecord>(conn)
		.await
		.optional()?
		.map(TryFrom::try_from)
		.transpose()
		.map_err(From::from)
}

#[tracing::instrument(skip(conn))]
pub async fn fetch_latest_block(conn: &mut DbConn) -> Result<Option<Block>, StoreError> {
	schema::block::table
		.left_join(schema::tx::table.on(schema::tx::block_height.eq(schema::block::height)))
		.select((schema::block::all_columns, &*TX_BZ_ARRAY_AGG))
		.group_by(schema::block::all_columns)
		.order(schema::block::height.desc())
		.first::<BlockWithDataRecord>(conn)
		.await
		.optional()?
		.map(TryFrom::try_from)
		.transpose()
		.map_err(From::from)
}

#[tracing::instrument(skip(conn))]
pub async fn fetch_block_by_height(
	conn: &mut DbConn,
	height: NonZeroU64,
) -> Result<Option<Block>, StoreError> {
	schema::block::table
		.left_join(schema::tx::table.on(schema::tx::block_height.eq(schema::block::height)))
		.filter(
			schema::block::height.eq(i64::try_from(height.get()).map_err(InvalidValueError::from)?),
		)
		.select((schema::block::all_columns, &*TX_BZ_ARRAY_AGG))
		.group_by(schema::block::all_columns)
		.first::<BlockWithDataRecord>(conn)
		.await
		.optional()?
		.map(TryFrom::try_from)
		.transpose()
		.map_err(From::from)
}

#[tracing::instrument(skip(conn))]
pub async fn fetch_block_by_block_hash(
	conn: &mut DbConn,
	block_hash: &Sha256,
) -> Result<Option<Block>, StoreError> {
	schema::block::table
		.left_join(schema::tx::table.on(schema::tx::block_height.eq(schema::block::height)))
		.filter(schema::block::block_hash.eq(block_hash.get()))
		.select((schema::block::all_columns, &*TX_BZ_ARRAY_AGG))
		.group_by(schema::block::all_columns)
		.first::<BlockWithDataRecord>(conn)
		.await
		.optional()?
		.map(TryFrom::try_from)
		.transpose()
		.map_err(From::from)
}

#[tracing::instrument(skip(conn))]
pub async fn fetch_tx_by_block_height_and_tx_idx_in_block(
	conn: &mut DbConn,
	height: NonZeroU64,
	tx_idx_in_block: u64,
) -> Result<Option<Tx>, StoreError> {
	let height = i64::try_from(height.get()).map_err(InvalidValueError::from)?;
	let tx_idx_in_block = i64::try_from(tx_idx_in_block).map_err(InvalidValueError::from)?;

	let Some(tx) = schema::tx::table
		.select(schema::tx::all_columns)
		.filter(schema::tx::block_height.eq(height))
		.filter(schema::tx::tx_idx_in_block.eq(tx_idx_in_block))
		.first(conn)
		.await
		.optional()?
	else {
		return Ok(None);
	};

	let signatures = fetch_signatures(conn, height, tx_idx_in_block).await?;
	let fee = fetch_fees(conn, height, tx_idx_in_block).await?;
	let msgs = fetch_msgs(conn, height, tx_idx_in_block).await?;

	TxWithDetailsRecord::builder()
		.tx(tx)
		.signatures(signatures.into_iter().map(SignatureRecord::into_bytes).collect())
		.msgs(msgs.into_iter().map(From::from).collect())
		.fees(fee.iter().map(TryFrom::try_from).collect::<Result<_, _>>()?)
		.build()
		.try_into()
		.map(Some)
		.map_err(From::from)
}

#[tracing::instrument(skip(conn))]
pub async fn fetch_tx_by_tx_hash(
	conn: &mut DbConn,
	tx_hash: &Sha256,
) -> Result<Option<Tx>, StoreError> {
	let Some(tx) = schema::tx::table
		.select(schema::tx::all_columns)
		.filter(schema::tx::tx_hash.eq(tx_hash.get()))
		.first::<TxRecord>(conn)
		.await
		.optional()?
	else {
		return Ok(None);
	};

	let height = tx.block_height();
	let tx_idx_in_block = tx.tx_idx_in_block();

	let signatures = fetch_signatures(conn, height, tx_idx_in_block).await?;
	let fee = fetch_fees(conn, height, tx_idx_in_block).await?;
	let msgs = fetch_msgs(conn, height, tx_idx_in_block).await?;

	TxWithDetailsRecord::builder()
		.tx(tx)
		.signatures(signatures.into_iter().map(SignatureRecord::into_bytes).collect())
		.msgs(msgs.into_iter().map(From::from).collect())
		.fees(fee.iter().map(TryFrom::try_from).collect::<Result<_, _>>()?)
		.build()
		.try_into()
		.map(Some)
		.map_err(From::from)
}

fn process_new_records_from_txs<'a>(
	txs: &'a [Tx],
	new_tx_records: &mut Vec<NewTxRecord<'a>>,
	new_signature_records: &mut Vec<NewSignatureRecord<'a>>,
	new_fee_records: &mut Vec<NewFeeRecord<'a>>,
	new_msg_records: &mut Vec<NewMsgRecord<'a>>,
) -> Result<(), InvalidValueError> {
	for tx in txs {
		new_tx_records.push(tx.try_into()?);

		let block_height = tx.block_height().get().try_into()?;
		let tx_idx_in_block = tx.tx_idx_in_block().try_into()?;

		for (idx, signature) in tx.signatures().iter().enumerate() {
			let signature_record = NewSignatureRecord::builder()
				.block_height(block_height)
				.tx_idx_in_block(tx_idx_in_block)
				.signature_idx_in_tx(idx.try_into()?)
				.bz(signature)
				.build();

			new_signature_records.push(signature_record);
		}

		for (idx, coin) in tx.fees().map(Fees::get).into_iter().flatten().enumerate() {
			let fee_record = NewFeeRecord::builder()
				.block_height(block_height)
				.tx_idx_in_block(tx_idx_in_block)
				.fee_idx_in_tx(idx.try_into()?)
				.amount(coin.amount.into())
				.denom(coin.denom.as_ref())
				.build();

			new_fee_records.push(fee_record);
		}

		for (idx, msg) in tx.msgs().get().iter().enumerate() {
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

async fn fetch_signatures(
	conn: &mut DbConn,
	height: i64,
	tx_idx_in_block: i64,
) -> Result<Vec<SignatureRecord>, diesel::result::Error> {
	schema::signature::table
		.select((schema::signature::bz,))
		.filter(schema::signature::block_height.eq(height))
		.filter(schema::signature::tx_idx_in_block.eq(tx_idx_in_block))
		.order(schema::signature::signature_idx_in_tx.asc())
		.load(conn)
		.await
}

async fn fetch_fees(
	conn: &mut DbConn,
	height: i64,
	tx_idx_in_block: i64,
) -> Result<Vec<FeeRecord>, diesel::result::Error> {
	schema::fee::table
		.select((schema::fee::amount, schema::fee::denom))
		.filter(schema::fee::block_height.eq(height))
		.filter(schema::fee::tx_idx_in_block.eq(tx_idx_in_block))
		.order(schema::fee::fee_idx_in_tx.asc())
		.load(conn)
		.await
}

async fn fetch_msgs(
	conn: &mut DbConn,
	height: i64,
	tx_idx_in_block: i64,
) -> Result<Vec<MsgRecord>, diesel::result::Error> {
	schema::msg::table
		.select((schema::msg::type_url, schema::msg::value))
		.filter(schema::msg::block_height.eq(height))
		.filter(schema::msg::tx_idx_in_block.eq(tx_idx_in_block))
		.order(schema::msg::msg_idx_in_tx.asc())
		.load(conn)
		.await
}

fn lo_hi_from_bounds<RB>(bounds: RB) -> Result<(NonZeroU64, NonZeroU64), StoreError>
where
	RB: RangeBounds<NonZeroU64>,
{
	let lo = match bounds.start_bound() {
		Bound::Included(included) => *included,
		Bound::Excluded(excluded) => excluded.checked_add(1).ok_or(StoreError::Arithmetic)?,
		Bound::Unbounded => NonZeroU64::MIN,
	};

	let hi = match bounds.end_bound() {
		Bound::Included(included) => *included,
		Bound::Excluded(excluded) => {
			NonZeroU64::new(excluded.get() - 1).ok_or(StoreError::Arithmetic)?
		},
		Bound::Unbounded => NonZeroU64::MIN,
	};

	Ok((lo, hi))
}
