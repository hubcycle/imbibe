pub use diesel_async::pooled_connection::deadpool::PoolError;

#[cfg(feature = "database")]
pub use self::database::*;

#[cfg(feature = "database")]
mod database {
	use diesel_async::{
		AsyncPgConnection,
		pooled_connection::{
			AsyncDieselConnectionManager,
			deadpool::{BuildError, Object, Pool},
		},
	};

	pub type DbPool = Pool<AsyncPgConnection>;
	pub type DbConn = Object<AsyncPgConnection>;

	#[tracing::instrument(skip(url))]
	pub async fn establish_pool<U>(
		url: U,
		max_size: core::num::NonZeroUsize,
	) -> Result<DbPool, BuildError>
	where
		String: From<U>,
	{
		Pool::builder(AsyncDieselConnectionManager::new(url)).max_size(max_size.get()).build()
	}
}
