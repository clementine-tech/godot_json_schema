use super::*;

#[derive(Debug, Clone)]
pub enum ClassSource {
	Script { script: Gd<Script>, id: ClassId },
	Engine(StringName),
}

impl ClassSource {
	pub fn to_reference(&self) -> JRef {
		let class_name = match self {
			ClassSource::Script { id, .. } => id.definition_name(),
			ClassSource::Engine(string_name) => string_name.to_string(),
		};
		
		JRef::new(class_name)
	}
	
	pub fn id(&self) -> ClassId {
		match self {
			ClassSource::Script { id, .. } => id.clone(),
			ClassSource::Engine(string_name) => ClassId::Name(string_name.clone()),
		}
	}
	
	pub fn from_class_name(class_name: impl Into<StringName>) -> Result<Self> {
		let class_name = class_name.into();
		
		if ClassDb::singleton().class_exists(class_name.clone()) {
			Ok(Self::Engine(class_name))
		} else if let Ok(script) = find_script(class_name.clone()) {
			Ok(Self::Script { script, id: ClassId::Name(class_name) })
		} else {
			bail!("Expected class \"{class_name}\" to be in either `ClassDb` or `ProjectSettings`.");
		}
	}
	
	pub fn from_script(script: Gd<Script>) -> Self {
		let global_name = script.get_global_name();
		
		let id = if global_name.is_empty() {
			ClassId::Script(script.clone())
		} else {
			ClassId::Name(global_name)
		};
		
		Self::Script { script, id }
	}

	pub fn fetch_property_list(&self, definitions: &mut BTreeMap<String, Definition>) -> Result<BTreeMap<String, Type>> {
		match self {
			ClassSource::Script { script, .. } => {
				let properties_dict = script.clone().get_script_property_list();

				properties_dict
					.iter_shared()
					.filter_map(|dict| {
						// Skip the `file_name` property
						if let Ok(maybe_file_name) = try_get::<String>(&dict, "name")
							&& maybe_file_name.ends_with(".gd") {
							None
						} else {
							Some(dict)
						}
					})
					.map(|dict| PropertyWrapper::try_from(dict)?.eval_type(definitions))
					.try_collect()
			}
			ClassSource::Engine(class_name) => ClassDb::singleton()
				.class_get_property_list(class_name.clone())
				.iter_shared()
				.map(|dict| PropertyWrapper::try_from(dict)?.eval_type(definitions))
				.try_collect(),
		}
	}
}

fn find_script(class_name: StringName) -> Result<Gd<Script>> {
	let class_list = ProjectSettings::singleton().get_global_class_list();

	for dict in class_list.iter_shared() {
		let retrieved_name = try_get::<StringName>(&dict, "class")?;

		if retrieved_name == class_name {
			let path = try_get::<GString>(&dict, "path")?;

			let resource = ResourceLoader::singleton()
				.load(path.clone())
				.ok_or_else(|| anyhow!("Expected gd_script file at path `{path}` to exist."))?;

			let gd_script = resource
				.try_cast::<Script>()
				.map_err(|err| anyhow!("{err:?}"))?;

			return Ok(gd_script);
		}
	}

	bail!("Expected class \"{class_name}\" to be in `ProjectSettings`.")
}