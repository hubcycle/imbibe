use std::collections::{HashMap, HashSet};

use prost_reflect::{
	DescriptorPool, ExtensionDescriptor, FieldDescriptor, Kind, MessageDescriptor, Value,
};

const SIGNER_OPTION_FQN: &str = "cosmos.msg.v1.signer";

pub fn find_signer_msgs(pool: &DescriptorPool) -> anyhow::Result<HashMap<String, Vec<String>>> {
	let Some(signer_ext_desc) = pool.get_extension_by_name(SIGNER_OPTION_FQN) else {
		println!("info: extension '{SIGNER_OPTION_FQN}' not found in the descriptor pool",);
		return Ok(HashMap::new());
	};

	let mut valid_msgs = HashMap::new();
	let mut visited = HashSet::new();

	pool.all_messages()
		.try_for_each(|msg| process_msg(&msg, &signer_ext_desc, &mut valid_msgs, &mut visited))?;

	Ok(valid_msgs)
}

fn process_msg(
	msg: &MessageDescriptor,
	signer_ext_desc: &ExtensionDescriptor,
	valid_msgs: &mut HashMap<String, Vec<String>>,
	visited: &mut HashSet<String>,
) -> anyhow::Result<()> {
	let full_name = msg.full_name();
	if visited.contains(full_name) {
		return Ok(());
	}

	visited.insert(full_name.into());

	let msg_options_dyn = msg.options();

	if !msg_options_dyn.has_extension(signer_ext_desc) {
		return Ok(());
	}

	let signer_fields: Vec<String> = msg_options_dyn
		.get_extension(signer_ext_desc)
		.as_list()
		.map(|fields| fields.iter().filter_map(Value::as_str).map(From::from).collect())
		.unwrap_or_default();

	if signer_fields.is_empty() {
		return Ok(());
	}

	let mut valid_fields = vec![];
	for field_name in signer_fields {
		let field = msg
			.get_field_by_name(&field_name)
			.ok_or_else(|| anyhow::anyhow!("field {field_name} not found in {full_name}"))?;

		if is_valid_field(&field, signer_ext_desc, msg, valid_msgs, visited)? {
			valid_fields.push(field_name);
		}
	}

	(!valid_fields.is_empty()).then(|| valid_msgs.insert(full_name.into(), valid_fields));

	Ok(())
}

fn is_valid_field(
	field: &FieldDescriptor,
	signer_ext_desc: &ExtensionDescriptor,
	parent_msg: &MessageDescriptor,
	valid_msgs: &mut HashMap<String, Vec<String>>,
	visited: &mut HashSet<String>,
) -> anyhow::Result<bool> {
	match field.kind() {
		Kind::String => Ok(true),
		Kind::Message(msg) => {
			process_msg(&msg, signer_ext_desc, valid_msgs, visited)?;
			Ok(valid_msgs.contains_key(msg.full_name()))
		},
		k => {
			eprintln!(
				"warning: field '{}' of {} in '{SIGNER_OPTION_FQN}' extension list is neither a string nor a message but of kind {k:?}",
				field.name(),
				parent_msg.full_name(),
			);
			Ok(false)
		},
	}
}
