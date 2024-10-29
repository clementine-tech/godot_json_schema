use super::*;

#[derive(Clone, Debug, Default)]
pub struct JObject {
	pub description: Option<String>,
	// If properties is empty, then the object is a Dictionary with any number of key/value pairs
	pub properties: BTreeMap<String, Type>,
}

impl JObject {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add_property(&mut self, key: impl Into<String>, ty: impl Into<Type>) {
		self.properties.insert(key.into(), ty.into());
	}

	pub fn with_properties(properties: impl Iterator<Item = (impl Into<String>, impl Into<Type>)>) -> Self {
		Self {
			description: None,
			properties: properties
				.map(|(k, v)| (k.into(), v.into()))
				.collect(),
		}
	}

	pub fn insert_variant_definitions(&self, fill_me: &mut Vec<VariantDefinition>) {
		for ty in self.properties.values() {
			ty.insert_variant_definitions(fill_me);
		}
	}
}

impl SerializeFields for JObject {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("type", "object")?;

		if !self.properties.is_empty() {
			map.serialize_entry("properties", &self.properties)?;
			map.serialize_entry("required", &self.properties.keys().collect::<Vec<_>>())?;
			map.serialize_entry("additionalProperties", &false)?;
		}

		Ok(())
	}
}

all_shared_impls!(JObject);