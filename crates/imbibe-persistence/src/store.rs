#[cfg(feature = "database")]
mod database;

mod error;

use diesel_async::RunQueryDsl;

pub use crate::record::error::InvalidValueError;

#[cfg(feature = "database")]
pub use self::database::*;

pub use self::error::StoreError;

#[cfg(feature = "database")]
pub async fn update_summaries(
	conn: &mut crate::pool::DbConn,
	window_sizes: &[core::num::NonZeroU64],
	upto: core::num::NonZeroU64,
	start: core::num::NonZeroU64,
) -> Result<(), StoreError> {
	let window_sizes: Vec<_> = window_sizes
		.iter()
		.map(|w| w.get())
		.map(i64::try_from)
		.collect::<Result<_, _>>()
		.map_err(InvalidValueError::from)?;

	diesel::select(sql::update_summaries_from_block(
		window_sizes,
		i64::try_from(upto.get()).map_err(InvalidValueError::from)?,
		i64::try_from(start.get()).map_err(InvalidValueError::from)?,
	))
	.execute(conn)
	.await?;

	Ok(())
}

mod sql {
	use diesel::sql_types::{Array, BigInt};

	diesel::define_sql_function! {
		fn update_summaries_from_block(
			windows: Array<BigInt>,
			current_block: BigInt,
			start_block: BigInt,
		)
	}
}
