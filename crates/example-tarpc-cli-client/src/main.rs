use core::num::NonZeroU64;

use clap::{Parser, Subcommand};
use imbibe_domain::Sha256;
use imbibe_querier::tarpc::QueryClient;
use tarpc::{client::Config, context, tokio_serde::formats::Json};

#[derive(Parser)]
#[command(name = "tarpc-example-cli")]
#[command(about = "tarpc-example-cli", long_about = None,)]
#[command(version = "0.0.1")]
struct Cli {
	#[arg(
		global = true,
		long = "tarpc-server",
		default_value = "localhost:18181",
		help = "address of the tarpc server [default: localhost:18181]"
	)]
	tarpc_server: String,

	#[command(subcommand)]
	command: Command,
}

#[derive(Subcommand)]
enum Command {
	BlockByHeight {
		/// must be a positive integer.
		height: NonZeroU64,
	},
	BlockByBlockHash {
		/// is a hex string SHA256 hash.
		block_hash: String,
	},
	TxByHeightAndTxIdx {
		/// must be a positive integer.
		height: NonZeroU64,

		/// is the index of tx in the block. Must be a non-negative integer.
		tx_idx_in_block: u64,
	},
	TxByTxHash {
		/// is a hex string SHA256 hash.
		tx_hash: String,
	},
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let cli = Cli::parse();

	let server_addr = cli.tarpc_server;

	let transport = tarpc::serde_transport::tcp::connect(server_addr, Json::default);
	let client = QueryClient::new(Config::default(), transport.await?).spawn();

	match cli.command {
		Command::BlockByHeight { height } => {
			client
				.block_by_height(context::current(), height)
				.await?
				.inspect(|block| println!("block at height {height}:\n{block:#?}"))?;
		},
		Command::BlockByBlockHash { block_hash } => {
			let hash = Sha256::new(const_hex::decode_to_array(&block_hash)?);
			client
				.block_by_block_hash(context::current(), hash)
				.await?
				.inspect(|block| println!("block at height {block_hash}:\n{block:#?}"))?;
		},
		Command::TxByHeightAndTxIdx { height, tx_idx_in_block } => {
			client
				.tx_by_block_height_and_tx_idx_in_block(context::current(), height, tx_idx_in_block)
				.await?
				.inspect(|tx| {
					println!(
						"tx at height {height} and with index {tx_idx_in_block} in block\n{tx:#?}"
					)
				})?;
		},
		Command::TxByTxHash { tx_hash } => {
			let hash = Sha256::new(const_hex::decode_to_array(&tx_hash)?);
			client
				.tx_by_tx_hash(context::current(), hash)
				.await?
				.inspect(|tx| println!("tx with tx hash {tx_hash}:\n{tx:#?}"))?;
		},
	};

	Ok(())
}
