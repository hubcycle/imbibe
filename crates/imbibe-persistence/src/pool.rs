pub use diesel_async::pooled_connection::deadpool::PoolError;

use core::num::NonZeroUsize;

use diesel_async::{
	AsyncPgConnection,
	pooled_connection::{
		AsyncDieselConnectionManager,
		deadpool::{BuildError, Object, Pool},
	},
};
use url::Url;

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConn = Object<AsyncPgConnection>;

pub async fn establish_pool(url: Url, max_size: NonZeroUsize) -> Result<DbPool, BuildError> {
	let pool =
		Pool::builder(AsyncDieselConnectionManager::new(url)).max_size(max_size.get()).build()?;

	Ok(pool)
}
