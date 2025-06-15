use std::{collections::HashSet, env, path::PathBuf, sync::LazyLock};

pub static OUT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
	const OUT_DIR_ENV_VAR: &str = "OUT_DIR";
	env::var(OUT_DIR_ENV_VAR)
		.inspect_err(|e| eprintln!("env var '{OUT_DIR_ENV_VAR}' must be set: {}", e))
		.unwrap()
		.into()
});

#[cfg(feature = "custom")]
pub static PROTO_SRC_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
	const PROTO_SRC_DIR_ENV_VAR: &str = "PROTO_SRC_DIR";
	env::var(PROTO_SRC_DIR_ENV_VAR)
		.inspect_err(|e| eprintln!("env var '{PROTO_SRC_DIR_ENV_VAR}' must be set: {}", e))
		.unwrap()
		.into()
});

pub static PROTO_EXPORT_DIR: LazyLock<PathBuf> = LazyLock::new(|| OUT_DIR.join("proto_export"));

pub static CODE_GEN_DIR: LazyLock<PathBuf> = LazyLock::new(|| OUT_DIR.join("code_gen"));

pub static EXCLUDED_PROTOS: LazyLock<HashSet<PathBuf>> = LazyLock::new(|| {
	[PathBuf::from("cosmos").join("staking").join("v1beta1").join("authz.proto")]
		.into_iter()
		.map(|p| PROTO_EXPORT_DIR.join(p))
		.collect()
});
