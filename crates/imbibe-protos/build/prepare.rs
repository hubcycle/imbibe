use std::{fs, path::Path};

use anyhow::Context;

#[allow(dead_code)]
#[cfg(feature = "protogen")]
const BUF_BIN: &str = "buf";

#[cfg(feature = "protogen")]
pub fn prepare_directory<P>(dir: P) -> anyhow::Result<()>
where
	P: AsRef<Path>,
{
	let dir = dir.as_ref();

	if dir.try_exists().context(format!("failed inspecting {}", dir.display()))? {
		fs::remove_dir_all(dir)
	} else {
		fs::create_dir_all(dir)
	}?;

	Ok(())
}

#[cfg(feature = "protogen")]
pub fn exclude_protos() -> anyhow::Result<()> {
	for proto in crate::global::EXCLUDED_PROTOS.iter() {
		proto.try_exists()?.then(|| fs::remove_file(proto)).transpose()?;
	}

	Ok(())
}

#[cfg(feature = "custom")]
pub fn setup_proto_src_watching_and_deps<P>(proto_src_dir: P) -> anyhow::Result<()>
where
	P: AsRef<Path>,
{
	let proto_src_dir = proto_src_dir.as_ref();

	proto_src_dir.try_exists()?.then(|| watch_dir_recursively(proto_src_dir)).is_none().then(
		|| {
			println!(
				"cargo:warning=proto src directory '{}' not found",
				proto_src_dir.display()
			);
		},
	);

	let buf_yaml = proto_src_dir.join("buf.yaml");
	let buf_lock = proto_src_dir.join("buf.lock");

	println!("cargo:rerun-if-changed={}", buf_yaml.display());

	buf_lock
		.try_exists()?
		.then(|| println!("cargo:rerun-if-changed={}", buf_lock.display()))
		.ok_or_else(|| anyhow::anyhow!("buf.lock must exist, run '{BUF_BIN} dep update'"))
}

#[cfg(feature = "custom")]
pub fn run_buf_export<P1, P2>(src_dir: P1, export_dir: P2) -> anyhow::Result<()>
where
	P1: AsRef<Path>,
	P2: AsRef<Path>,
{
	println!("buf export to {}", src_dir.as_ref().display());
	let buf_status = std::process::Command::new(BUF_BIN)
		.arg("export")
		.arg(src_dir.as_ref())
		.arg("--output")
		.arg(export_dir.as_ref())
		.stdout(std::process::Stdio::inherit())
		.stderr(std::process::Stdio::inherit())
		.status()
		.with_context(|| {
			format!("buf command failed. Ensure '{BUF_BIN}' is installed and in PATH")
		})?;

	if !buf_status.success() {
		anyhow::bail!("buf export failed with status: {}", buf_status);
	}

	println!("buf export finished successfully");

	Ok(())
}

#[cfg(feature = "cosmos")]
pub fn buf_export_cosmos_sdk<P>(proto_export_dir: P) -> anyhow::Result<()>
where
	P: AsRef<Path>,
{
	let buf_status = std::process::Command::new(BUF_BIN)
		.arg("export")
		.arg("buf.build/cosmos/cosmos-sdk")
		.arg("--output")
		.arg(proto_export_dir.as_ref())
		.stdout(std::process::Stdio::inherit())
		.stderr(std::process::Stdio::inherit())
		.status()
		.with_context(|| {
			format!("buf command failed. Ensure '{BUF_BIN}' is installed and in PATH")
		})?;

	if !buf_status.success() {
		anyhow::bail!("buf export command failed with status: {}", buf_status);
	}

	println!("buf export of cosmos-sdk finished successfully");

	Ok(())
}

#[cfg(feature = "ethsecp256k1")]
pub fn buf_export_ethsecp256k1<P>(proto_export_dir: P) -> anyhow::Result<()>
where
	P: AsRef<Path>,
{
	let buf_status = std::process::Command::new(BUF_BIN)
		.arg("export")
		.arg("buf.build/evmos/ethermint")
		.arg("--output")
		.arg(proto_export_dir.as_ref())
		.arg("--path")
		.arg("ethermint/crypto/v1/ethsecp256k1/keys.proto")
		.stdout(std::process::Stdio::inherit())
		.stderr(std::process::Stdio::inherit())
		.status()
		.with_context(|| {
			format!("buf command failed. Ensure '{BUF_BIN}' is installed and in PATH")
		})?;

	if !buf_status.success() {
		anyhow::bail!("buf export command failed with status: {}", buf_status);
	}

	println!("buf export of cosmos-sdk finished successfully");

	Ok(())
}

#[cfg(feature = "custom")]
fn watch_dir_recursively<P>(path: P)
where
	P: AsRef<Path>,
{
	use walkdir::WalkDir;

	let path = path.as_ref();

	println!("cargo:rerun-if-changed={}", path.display());

	for entry in WalkDir::new(path) {
		let _ = entry
			.inspect(|entry| {
				if entry.file_type().is_file() {
					println!("cargo:rerun-if-changed={}", entry.path().display());
				}
			})
			.inspect_err(|err| {
				eprintln!(
					"Warning: error walking directory {}: {}, rerun tracking might be incomplete",
					path.display(),
					err
				);
			});
	}
}
