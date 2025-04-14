use core::{cmp::Ordering, num::NonZeroU64};

use anyhow::Context;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
use futures::{StreamExt, TryStreamExt};

use crate::domain::block::Block;

use super::{DbConn, record::BlockRecord, schema};

pub async fn save_block(conn: &mut DbConn, block: Block) -> anyhow::Result<()> {
	let record = BlockRecord::try_from(block)?;

	diesel::insert_into(schema::block::table).values(record).execute(conn).await?;

	Ok(())
}

#[tracing::instrument(level = "info", skip_all)]
pub async fn save_blocks<I>(conn: &mut DbConn, blocks: I) -> anyhow::Result<()>
where
	I: Iterator<Item = Block>,
{
	let records: Vec<_> = blocks.map(BlockRecord::try_from).collect::<Result<_, _>>()?;

	diesel::insert_into(schema::block::table).values(records).execute(conn).await?;

	Ok(())
}

#[tracing::instrument(level = "info", skip(conn))]
pub async fn fetch_missing_block_heights(
	conn: &mut DbConn,
	upto: NonZeroU64,
) -> anyhow::Result<impl Iterator<Item = NonZeroU64>> {
	use schema::block::dsl::{block, height};

	let existing_heights: Vec<_> = block
		.select(height)
		.filter(height.lt(i64::try_from(upto.get())?))
		.order(height.asc())
		.load_stream::<i64>(conn)
		.await?
		.map(|h| NonZeroU64::new(h?.try_into()?).context("invalid height"))
		.try_collect()
		.await?;

	tracing::info!("existing heights = {existing_heights:?}");

	let all_heights = (1..upto.get()).flat_map(NonZeroU64::new);

	Ok(subtract_sorted_iter(
		all_heights,
		existing_heights.into_iter(),
	))
}

fn subtract_sorted_iter<A, B, T>(mut a: A, mut b: B) -> impl Iterator<Item = T>
where
	A: Iterator<Item = T>,
	B: Iterator<Item = T>,
	T: Ord,
{
	let mut next_a = a.next();
	let mut next_b = b.next();

	std::iter::from_fn(move || {
		while let Some(ref va) = next_a {
			match &next_b {
				Some(vb) => match va.cmp(vb) {
					Ordering::Less => {
						let result = next_a.take();
						next_a = a.next();
						return result;
					},
					Ordering::Equal => {
						next_a = a.next();
						next_b = b.next();
					},
					Ordering::Greater => next_b = b.next(),
				},
				None => {
					let result = next_a.take();
					next_a = a.next();
					return result;
				},
			}
		}

		None
	})
}
