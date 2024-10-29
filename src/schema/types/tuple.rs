use super::*;

#[derive(Clone, Debug)]
pub struct JTuple {
	pub description: Option<String>,
	pub items: Vec<Type>,
}

impl SerializeFields for JTuple {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("type", "array")?;
		map.serialize_entry("prefixItems ", &self.items)
	}
}

impl JTuple {
	pub fn new(items: impl IntoIterator<Item = impl Into<Type>>) -> Self {
		Self {
			description: None,
			items: items.into_iter().map(Into::into).collect(),
		}
	}

	pub fn insert_variant_definitions(&self, fill_me: &mut Vec<VariantDefinition>) {
		for ty in &self.items {
			ty.insert_variant_definitions(fill_me);
		}
	}
}

all_shared_impls!(JTuple);