use core::num::NonZeroU64;

use diesel::{prelude::QueryableByName, sql_types::BigInt};
use diesel_async::RunQueryDsl;
use futures::{Stream, StreamExt, TryStreamExt};

use crate::domain::block::Block;

use super::{DbConn, record::BlockRecord, schema};

#[tracing::instrument(skip_all)]
pub async fn save_block(conn: &mut DbConn, block: Block) -> anyhow::Result<()> {
	let record = BlockRecord::try_from(block)?;

	diesel::insert_into(schema::block::table).values(record).execute(conn).await?;

	Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn save_blocks<I>(conn: &mut DbConn, blocks: I) -> anyhow::Result<()>
where
	I: Iterator<Item = Block>,
{
	let records: Vec<_> = blocks.map(BlockRecord::try_from).collect::<Result<_, _>>()?;

	diesel::insert_into(schema::block::table).values(records).execute(conn).await?;

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
