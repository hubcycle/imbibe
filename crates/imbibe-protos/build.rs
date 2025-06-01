#[cfg(feature = "protogen")]
#[path = "build/codegen.rs"]
mod codegen;

#[cfg(feature = "protogen")]
#[path = "build/global.rs"]
mod global;

#[cfg(feature = "protogen")]
#[path = "build/prepare.rs"]
mod prepare;

fn main() -> anyhow::Result<()> {
	#[cfg(feature = "protogen")]
	{
		prepare::prepare_directory(&*global::PROTO_EXPORT_DIR)?;
		prepare::prepare_directory(&*global::CODE_GEN_DIR)?;
	}

	#[cfg(feature = "custom")]
	{
		prepare::setup_proto_src_watching_and_deps(&*global::PROTO_SRC_DIR)?;
		prepare::run_buf_export(&*global::PROTO_SRC_DIR, &*global::PROTO_EXPORT_DIR)?;
	}

	#[cfg(feature = "cosmos")]
	prepare::buf_export_cosmos_sdk(&*global::PROTO_EXPORT_DIR)?;

	#[cfg(feature = "ethsecp256k1")]
	prepare::buf_export_ethsecp256k1(&*global::PROTO_EXPORT_DIR)?;

	#[cfg(feature = "protogen")]
	{
		prepare::exclude_protos()?;
		codegen::compile_protos(&*global::PROTO_EXPORT_DIR, &*global::CODE_GEN_DIR)?;
		codegen::generate_mod_rs(&*global::CODE_GEN_DIR)?;
	}

	Ok(())
}
