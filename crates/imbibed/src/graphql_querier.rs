use async_graphql::{EmptyMutation, EmptySubscription, Schema, http::GraphiQLSource};
use async_graphql_axum::GraphQL;
use axum::{Router, response::Html, routing};
use imbibe_persistence::pool::DbPool;
use imbibe_querier::{graphql::QueryRoot, server::Querier};
use tokio::net::{TcpListener, ToSocketAddrs};

pub async fn run<A>(pool: DbPool, sock_addr: A) -> anyhow::Result<()>
where
	A: ToSocketAddrs,
{
	let query_root = QueryRoot::builder().querier(Querier::builder().pool(pool).build()).build();

	let schema = Schema::build(query_root, EmptyMutation, EmptySubscription).finish();

	let app = Router::new().route(
		"/",
		routing::get(graphiql).post_service(GraphQL::new(schema)),
	);

	let listener = TcpListener::bind(sock_addr).await?;

	tracing::info!(
		"querier graphql listening port {}",
		listener.local_addr()?.port()
	);

	axum::serve(listener, app).await?;

	Ok(())
}

async fn graphiql() -> Html<String> {
	Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
