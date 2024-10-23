use super::*;

#[derive(Clone, Debug)]
pub struct RootSchema {
	pub defs: BTreeMap<String, Definition>,
	pub base: JClass,
}

impl RootSchema {
	pub fn generate(source: ClassSource) -> Result<RootSchema> {
		let mut defs = BTreeMap::new();
		let base = JClass::generate(source, &mut defs)?;

		Ok(RootSchema {
			defs,
			base,
		})
	}

	pub fn add_definition(&mut self, name: impl Into<String>, definition: impl Into<Definition>) {
		self.defs.insert(name.into(), definition.into());
	}

	pub fn add_class(&mut self, class: JClass) {
		self.add_definition(class.source.id().definition_name(), class);
	}

	pub fn instantiate(&self, hydrate_with: &Map<String, Value>) -> Result<Gd<Object>> {
		self.base.instantiate(&self.defs, hydrate_with)
	}

	pub fn to_json_compact(&self) -> serde_json::Result<String> {
		serde_json::to_string(self)
	}

	pub fn to_json_pretty(&self) -> serde_json::Result<String> {
		serde_json::to_string_pretty(self)
	}
}

impl Serialize for RootSchema {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut map = serializer.serialize_map(None)?;

		if let Some(description) = &self.base.description {
			map.serialize_entry("description", description)?;
		}

		map.serialize_entry("$schema", "http://json-schema.org/draft/2020-12/schema")?;
		
		let var_defs = {
			let mut vec = Vec::new();
			
			self.base.insert_variant_definitions(&mut vec);
			
			for def in self.defs.values() {
				def.insert_variant_definitions(&mut vec);
			}
			
			vec.retain(|def| !self.defs.contains_key(def.name()));
			
			vec
		};

		let all_defs = AllDefs {
			base_defs: &self.defs,
			var_defs,
		};
		
		map.serialize_entry("$defs", &all_defs)?;
		
		self.base.serialize_fields(&mut map)?;
		map.end()
	}
}

struct AllDefs<'a> {
	base_defs: &'a BTreeMap<String, Definition>,
	var_defs: Vec<VariantDefinition>,
}

impl<'a> Serialize for AllDefs<'a> {
	fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
		let mut map = serializer.serialize_map(None)?;
		
		for (name, def) in self.base_defs {
			map.serialize_entry(name, def)?;
		}
		
		for var_def in &self.var_defs {
			map.serialize_entry(var_def.name(), &var_def.source_definition())?;
		}
		
		map.end()
	}
} 