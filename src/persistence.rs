pub mod record;
pub mod store;

mod schema;

use diesel_async::{
	AsyncPgConnection,
	pooled_connection::{
		AsyncDieselConnectionManager,
		deadpool::{Object, Pool},
	},
};
use url::Url;

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConn = Object<AsyncPgConnection>;

pub async fn establish_pool(url: Url, max_size: usize) -> anyhow::Result<DbPool> {
	let pool = Pool::builder(AsyncDieselConnectionManager::new(url)).max_size(max_size).build()?;

	Ok(pool)
}
