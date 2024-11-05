use super::*;

#[derive(Clone, Debug)]
pub struct JClass {
	pub description: Option<String>,
	pub properties: BTreeMap<String, Type>,
	pub source: ClassSource,
}

impl JClass {
	pub fn add_property(&mut self, name: impl Into<String>, ty: impl Into<Type>) {
		self.properties.insert(name.into(), ty.into());
	}

	pub fn generate(source: ClassSource, insert_dependencies: &mut BTreeMap<String, Definition>) -> Result<Self> {
		let properties = source.fetch_property_list(insert_dependencies)?;

		Ok(Self {
			description: None,
			properties,
			source,
		})
	}

	pub fn instantiate(&self, defs: &BTreeMap<String, Definition>, property_values: &Map<String, Value>) -> Result<Gd<Object>> {
		let instance_var = match &self.source {
			// TODO: Check if script has a custom _init with parameters
			| ClassSource::ScriptNamed(script, _)
			| ClassSource::ScriptUnnamed(script) => script.clone().call("new", &[]),
			
			ClassSource::Engine(class_name) => ClassDb::singleton().instantiate(&class_name.clone()),
		};

		let mut gd = instance_var
			.try_to::<Gd<Object>>()
			.map_err(|err| anyhow!("{err:?}"))?;

		for (name, value) in property_values {
			let variant = {
				let ty = self
					.properties
					.get(name)
					.ok_or_else(|| anyhow!("Expected property \"{name}\" to be in `properties` map."))?;

				let schema = ty.resolve(defs)?;
				schema.instantiate(value, defs)?
			};
			
			gd.set(name, &variant);
		}

		Ok(gd)
	}

	pub fn insert_variant_definitions(&self, fill_me: &mut Vec<VariantDefinition>) {
		for ty in self.properties.values() {
			ty.insert_variant_definitions(fill_me);
		}
	}
}

impl SerializeFields for JClass {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("type", "object")?;
		map.serialize_entry("properties", &self.properties)?;
		map.serialize_entry("required", &self.properties.keys().collect::<Vec<_>>())?;
		map.serialize_entry("additionalProperties", &false)
	}
}

all_shared_impls!(JClass);