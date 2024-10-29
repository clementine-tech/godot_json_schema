use super::*;

#[derive(Clone, Debug, Default)]
pub struct Null {
	pub description: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Boolean {
	pub description: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Integer {
	pub description: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct Number {
	pub description: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct JString {
	pub description: Option<String>,
}

impl SerializeFields for Null {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("type", "null")
	}
}

impl SerializeFields for Boolean {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("type", "boolean")
	}
}

impl SerializeFields for Integer {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("type", "integer")
	}
}

impl SerializeFields for Number {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("type", "number")
	}
}

impl SerializeFields for JString {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("type", "string")
	}
}

all_shared_impls!(Null, Boolean, Integer, Number, JString);