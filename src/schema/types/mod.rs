use super::*;

pub use crate::schema::definition::*;
pub use array::*;
pub use object::*;
pub use primitives::*;
pub use reference::*;
pub use j_enum::*;
pub use tuple::*;
pub use godot_class::*;

pub mod primitives;
pub mod object;
pub mod array;
pub mod tuple;
pub mod j_enum;
pub mod reference;
pub mod shared_impls;
pub mod godot_class;

delegated_enum! {
	ENUM_OUT: {
		#[derive(Clone, Debug)]
		pub enum Type {
			Definition(Definition),
			Ref(JRef),
		}
	}
	
	DELEGATES: { 
		impl trait Serialize {
			[fn serialize<[S: Serializer]>(&self, serializer: S) -> Result<S::Ok, S::Error>]
		}
		
		impl {
			[pub fn add_description(&mut self, description: impl Into<String>)]
			[pub fn to_json_compact(&self) -> serde_json::Result<String>]
			[pub fn to_json_pretty(&self) -> serde_json::Result<String>]
		}
	}
}

impl Type {
	pub fn resolve<'a>(&'a self, defs: &'a BTreeMap<String, Definition>) -> Result<&'a Definition> {
		match self {
			Type::Definition(def) => Ok(def),
			Type::Ref(JRef { name, .. }) => defs
				.get(name)
				.ok_or_else(|| anyhow!("Expected definition \"{name}\" to be in `$defs` map."))
		}
	}

	pub fn untyped_array() -> Self {
		JArray::untyped().into()
	}

	pub fn array(item_ty: impl Into<Type>) -> Self {
		JArray::new(item_ty).into()
	}

	pub fn object(properties: impl Iterator<Item = (impl Into<String>, impl Into<Type>)>) -> Self {
		JObject::with_properties(properties).into()
	}

	pub fn string_enum(variants: impl Iterator<Item = (impl Into<String>, impl Into<i64>)>) -> Self {
		JEnum::new(variants).into()
	}

	pub fn insert_variant_definitions(&self, fill_me: &mut Vec<VariantDefinition>) {
		if let Type::Definition(def) = self {
			def.insert_variant_definitions(fill_me);
		}
	}
}

