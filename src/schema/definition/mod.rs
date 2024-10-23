use super::*;
pub use variant::*;
use macros::*;

pub mod json_conversion;
pub mod variant;
mod macros;

delegated_enum! {
	ENUM_OUT: {
		#[derive(Clone, Debug)]
		pub enum Definition {
			Null(Null),
			Boolean(Boolean),
			Integer(Integer),
			Number(Number),
			String(JString),
			Object(JObject),
			Array(JArray),
			Tuple(JTuple),
			Enum(JEnum),
			Class(JClass),
			Variant(VariantDefinition),
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

impl Definition {
	pub fn null() -> Definition { Null::default().into() }
	pub fn boolean() -> Definition { Boolean::default().into() }
	pub fn integer() -> Definition { Integer::default().into() }
	pub fn number() -> Definition { Number::default().into() }
	pub fn string() -> Definition { JString::default().into() }
	pub fn untyped_array() -> Definition { JArray::untyped().into() }
	pub fn dictionary() -> Definition { JObject::new().into() }
	
	pub fn array(item_ty: impl Into<Type>) -> Definition {
		JArray::new(item_ty).into()
	}
	
	pub fn object(properties: impl Iterator<Item = (impl Into<String>, impl Into<Type>)>) -> Definition {
		JObject::with_properties(properties).into()
	}
	
	pub fn string_enum(variants: impl Iterator<Item = (impl Into<String>, impl Into<i64>)>) -> Definition {
		JEnum::new(variants).into()
	}
	
	pub fn into_reference(self, name: impl Into<String>, defs: &mut BTreeMap<String, Definition>) -> JRef {
		let name = name.into();
		defs.insert(name.clone(), self);
		JRef::new(name)
	}
	
	pub fn insert_variant_definitions(&self, fill_me: &mut Vec<VariantDefinition>) {
		match self {
			Definition::Object(obj) => obj.insert_variant_definitions(fill_me),
			Definition::Array(arr) => arr.insert_variant_definitions(fill_me),
			Definition::Tuple(tuple) => tuple.insert_variant_definitions(fill_me),
			Definition::Class(class) => class.insert_variant_definitions(fill_me),
			Definition::Variant(var) => var.insert_variant_definitions(fill_me),
			_ => {}
		}
	}
}

pub trait FromJson: Sized {
	fn try_from_json(json: &Value) -> Result<Self>;
}

pub trait GetDefinition {
	fn get_definition() -> Definition;
}

pub fn json_type_of<T: GetDefinition>() -> Type {
	Type::Definition(T::get_definition())
}

pub fn definition_of<T: GetDefinition>() -> Definition {
	T::get_definition()
}