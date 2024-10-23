use crate::schema::shared_impls::{impl_add_description, impl_serialize, impl_to_json};
use super::*;

#[derive(Clone, Debug)]
pub struct JRef {
	pub description: Option<String>,
	pub name: String,
}

impl JRef {
	pub fn new(name: impl Into<String>) -> Self {
		Self {
			description: None,
			name: name.into(),
		}
	}
}

impl SerializeFields for JRef {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("$ref", &format!("#/$defs/{}", self.name))
	}
}

impl_add_description!(JRef);
impl_to_json!(JRef);
impl_serialize!(JRef);