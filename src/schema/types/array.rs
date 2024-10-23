use super::*;
use crate::schema::shared_impls::all_shared_impls;

#[derive(Clone, Debug)]
pub struct JArray {
	pub description: Option<String>,
	// If None, then each element can be of any type
	pub items_ty: Option<Box<Type>>,
}

impl SerializeFields for JArray {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("type", "array")?;

		if let Some(ty) = &self.items_ty {
			map.serialize_entry("items", ty)?;
		}

		Ok(())
	}
}

impl JArray {
	pub fn new(items_ty: impl Into<Type>) -> Self {
		Self {
			description: None,
			items_ty: Some(Box::new(items_ty.into())),
		}
	}

	pub const fn untyped() -> Self {
		Self {
			description: None,
			items_ty: None,
		}
	}

	pub fn insert_variant_definitions(&self, fill_me: &mut Vec<VariantDefinition>) {
		if let Some(ty) = &self.items_ty {
			ty.insert_variant_definitions(fill_me);
		}
	}
}

all_shared_impls!(JArray);