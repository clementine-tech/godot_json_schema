use super::*;
use godot::sys;
use godot::sys::{interface_fn, GodotFfi};
use itertools::Itertools;
use std::ptr;

pub fn raw_variant_definition(ty: VariantType) -> Option<Definition> {
	Some(match ty {
		VariantType::BOOL => definition_of::<bool>(),
		VariantType::INT => definition_of::<i32>(),
		VariantType::FLOAT => definition_of::<f32>(),
		VariantType::STRING => definition_of::<String>(),
		VariantType::DICTIONARY => definition_of::<Dictionary>(),
		other => VariantDefinition::try_from(other)
			.ok()
			.map(|def| def.into())?,
	})
}

pub fn raw_variant_from_json(value: &Value) -> Result<Variant> {
	Ok(match value {
		Value::Null => Variant::nil(),
		Value::Bool(bool) => bool.to_variant(),
		Value::Number(number) =>
			if let Some(int) = number.as_i64() {
				int.to_variant()
			} else if let Some(int) = number.as_u64() {
				int.to_variant()
			} else if let Some(float) = number.as_f64() {
				float.to_variant()
			} else {
				unreachable!()
			},
		Value::String(str) => str.to_variant(),
		Value::Array(vec) => {
			if vec.is_empty() {
				return Ok(VariantArray::new().to_variant());
			}

			let variants = vec
				.iter()
				.map(raw_variant_from_json)
				.try_collect::<_, Vec<Variant>, _>()?;

			let first_ty = variants[0].get_type();

			for var in variants.iter().skip(1) {
				if var.get_type() != first_ty {
					return Ok(VariantArray::from_iter(variants).to_variant());
				}
			}

			let typed_array = new_array_of_type(first_ty, None, None);

			for var in variants {
				typed_array.call("push_back", &[var]);
			}

			typed_array
		}
		Value::Object(properties) => properties
			.iter()
			.map(|(key, val)| Result::<(String, Variant)>::Ok((key.clone(), raw_variant_from_json(val)?)))
			.try_collect::<_, Dictionary, _>()?
			.to_variant(),
	})
}

impl Definition {
	pub fn variant_from_json(&self, value: &Value, defs: &BTreeMap<String, Definition>) -> Result<Variant> {
		match (self, value) {
			(Definition::Null(_), Value::Null) => Ok(Variant::nil()),
			(Definition::Boolean(_), Value::Bool(val)) => Ok(val.to_variant()),
			(Definition::Integer(_), Value::Number(number)) => Ok(
				if let Some(int) = number.as_i64() {
					int.to_variant()
				} else if let Some(int) = number.as_u64() {
					int.to_variant()
				} else {
					bail!("Expected integer, got float.");
				}
			),
			(Definition::Number(_), Value::Number(number)) => Ok(
				if let Some(int) = number.as_i64() {
					int.to_variant()
				} else if let Some(int) = number.as_u64() {
					int.to_variant()
				} else if let Some(float) = number.as_f64() {
					float.to_variant()
				} else {
					unreachable!()
				}
			),
			(Definition::String(_), Value::String(str)) => Ok(str.to_variant()),
			(Definition::Object(object), Value::Object(properties)) => {
				if object.properties.is_empty() {
					return Dictionary::try_from_json(value).map(|dict| dict.to_variant());
				}

				if object.properties.len() != properties.len() {
					bail!("Expected JSON object to have {} properties.\nGot: {}", object.properties.len(), properties.len());
				}

				let mut dict = Dictionary::new();

				for (name, ty) in &object.properties {
					let var = {
						let val = properties
							.get(name)
							.ok_or_else(|| anyhow!("Expected property \"{name}\" to be in `properties` map."))?;

						let schema = ty.resolve(defs)?;
						schema.variant_from_json(val, defs)?
					};

					dict.set(name.clone(), var);
				}

				Ok(dict.to_variant())
			}
			(Definition::Array(JArray { items_ty, .. }), Value::Array(vec)) => {
				if let Some(ty) = items_ty {
					let array = new_array_from_def(ty.resolve(defs)?)?;

					for json in vec {
						let var = {
							let schema = ty.resolve(defs)?;
							schema.variant_from_json(json, defs)?
						};
						
						array.call("push_back", &[var]);
					}

					Ok(array)
				} else {
					let mut array = VariantArray::new();
					
					for json in vec {
						array.push(raw_variant_from_json(json)?);
					}

					Ok(array.to_variant())
				}
			}
			(Definition::Tuple(JTuple { items, .. }), Value::Array(vec)) => {
				if items.len() != vec.len() {
					bail!("Expected JSON array to have {} elements.\nGot: {}", items.len(), vec.len());
				}

				let mut array = VariantArray::new();

				for (ty, json) in items.iter().zip(vec) {
					let var = {
						let schema = ty.resolve(defs)?;
						schema.variant_from_json(json, defs)?
					};

					array.push(var);
				}

				Ok(array.to_variant())
			}
			(Definition::Enum(JEnum { variants, .. }), Value::String(string)) => {
				if let Some(int_value) = variants.get(string) {
					Ok(int_value.to_variant())
				} else {
					bail!("Expected one of \"{}\".\nGot: {string}.", variants.keys().join(", "));
				}
			}
			(Definition::Class(class), Value::Object(properties)) => {
				Ok(class.instantiate(defs, properties)?.to_variant())
			}
			(Definition::Variant(variant_def), value) => {
				variant_def.var_from_json(value)
			}
			(Definition::Null(_), _) => bail!("Expected null, got: {value:?}"),
			(Definition::Boolean(_), _) => bail!("Expected boolean, got: {value:?}"),
			(Definition::Integer(_), _) => bail!("Expected integer, got: {value:?}"),
			(Definition::Number(_), _) => bail!("Expected number, got: {value:?}"),
			(Definition::String(_), _) => bail!("Expected string, got: {value:?}"),
			(Definition::Array(_), _) => bail!("Expected array, got: {value:?}"),
			(Definition::Object(_), _) => bail!("Expected object, got: {value:?}"),
			(Definition::Tuple(_), _) => bail!("Expected tuple, got: {value:?}"),
			(Definition::Enum(_), _) => bail!("Expected enum, got: {value:?}"),
			(Definition::Class(_), _) => bail!("Expected class, got: {value:?}"),
		}
	}
}

fn new_array_from_def(ty: &Definition) -> Result<Variant> {
	let (variant_type, class_name, script) =
		match ty {
			Definition::Class(class) => {
				match &class.source {
					| ClassSource::ScriptNamed(script, _)
					| ClassSource::ScriptUnnamed(script) => (VariantType::OBJECT, None, Some(script)),
					
					ClassSource::Engine(class_name) => (VariantType::OBJECT, Some(class_name), None),
				}
			}
			Definition::Boolean(_) => (VariantType::BOOL, None, None),
			Definition::Integer(_) => (VariantType::INT, None, None),
			Definition::Number(_) => (VariantType::FLOAT, None, None),
			Definition::String(_) => (VariantType::STRING, None, None),
			Definition::Array(_) => (VariantType::ARRAY, None, None),
			Definition::Object(_) => (VariantType::DICTIONARY, None, None),
			Definition::Null(_) => (VariantType::NIL, None, None),
			Definition::Enum(_) => (VariantType::INT, None, None),
			Definition::Tuple(_) => (VariantType::ARRAY, None, None),
			Definition::Variant(var_def) => (var_def.variant_type(), None, None),
		};

	Ok(new_array_of_type(variant_type, class_name, script))
}

fn new_array_of_type(
	variant_type: VariantType,
	class_name: Option<&StringName>,
	script: Option<&Gd<Script>>,
) -> Variant {
	let mut array = unsafe {
		VariantArray::new_with_uninit(|self_ptr| {
			let ctor = sys::builtin_fn!(array_construct_default);
			ctor(self_ptr, ptr::null_mut())
		})
	};

	#[allow(unused_assignments)]
	let mut empty_string_name = None;

	let class_name = if let Some(name) = class_name {
		name.string_sys()
	} else if let Some(script) = &script {
		empty_string_name = Some(script.get_instance_base_type());
		empty_string_name.as_ref().unwrap().string_sys()
	} else {
		empty_string_name = Some(StringName::default());
		// as_ref() crucial here -- otherwise the StringName is dropped.
		empty_string_name.as_ref().unwrap().string_sys()
	};

	let script = if let Some(script) = script {
		script.to_variant()
	} else {
		Variant::nil()
	};

	unsafe {
		interface_fn!(array_set_typed)(
			array.sys_mut(),
			variant_type.sys(),
			class_name,
			script.var_sys(),
		);
	}

	array.to_variant()
}