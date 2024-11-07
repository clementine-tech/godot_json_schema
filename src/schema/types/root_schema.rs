use super::*;

#[derive(Clone, Debug)]
pub struct RootSchema {
	pub defs: BTreeMap<String, Definition>,
	pub base: Definition,
}

impl RootSchema {
	pub fn from_class(source: ClassSource) -> Result<RootSchema> {
		let mut defs = BTreeMap::new();
		let base = Definition::from_class(source, &mut defs)?;

		Ok(RootSchema {
			defs,
			base,
		})
	}

	pub fn from_type_info(property: PropertyTypeInfo) -> Result<Self> {
		let mut defs = BTreeMap::new();
		let base_ty = property.eval_type(&mut defs)?;

		let base = match base_ty {
			Type::Definition(Definition::Variant(var_def)) => var_def.source_definition(),
			Type::Definition(base) => base,
			Type::Ref(JRef { name, .. }) => defs
				.remove(&name)
				.ok_or_else(|| anyhow!("Expected definition \"{name}\" to be in `$defs` map."))?,
		};

		Ok(RootSchema {
			defs,
			base,
		})
	}

	pub fn add_definition(&mut self, name: impl Into<String>, definition: impl Into<Definition>) {
		self.defs.insert(name.into(), definition.into());
	}

	pub fn add_class(&mut self, class: JClass) {
		self.add_definition(class.source.definition_name(), class);
	}

	pub fn instantiate(&self, value: &Value) -> Result<Variant> {
		self.base.instantiate(value, &self.defs)
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

		if let Some(description) = self.base.description() {
			map.serialize_entry("description", description)?;
		}

		map.serialize_entry("$schema", "https://json-schema.org/draft/2020-12/schema")?;

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
		
		match &self.base {
			Definition::Class(class) => class.serialize_fields(&mut map)?,
			Definition::Object(obj) => obj.serialize_fields(&mut map)?,
			not_class => {
				let class = Builder::object()
					.property("value", not_class.clone())
					.done();
				
				class.serialize_fields(&mut map)?;
			}
		}
		
		map.end()
	}
}

struct AllDefs<'a> {
	base_defs: &'a BTreeMap<String, Definition>,
	var_defs: Vec<VariantDefinition>,
}

impl Serialize for AllDefs<'_> {
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