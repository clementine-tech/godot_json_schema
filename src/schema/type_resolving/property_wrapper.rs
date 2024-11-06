use super::*;

pub struct PropertyTypeInfo {
	pub variant_type: VariantType,
	pub class_name: StringName,
	pub property_name: String,
	pub hint: PropertyHint,
	pub hint_string: String,
	pub usage: PropertyUsageFlags,
}

impl TryFrom<Dictionary> for PropertyTypeInfo {
	type Error = anyhow::Error;

	fn try_from(dict: Dictionary) -> std::result::Result<Self, Self::Error> {
		Ok(PropertyTypeInfo {
			property_name: try_get(&dict, "name")?,
			variant_type: try_get(&dict, "type")?,
			class_name: try_get(&dict, "class_name")?,
			hint: try_get(&dict, "hint")?,
			hint_string: try_get(&dict, "hint_string")?,
			usage: try_get(&dict, "usage")?,
		})
	}
}

impl PropertyTypeInfo {
	pub fn eval_type(&self, defs: &mut BTreeMap<String, Definition>) -> Result<Type> {
		let schema = match self.variant_type {
			VariantType::INT if self.usage.is_set(PropertyUsageFlags::CLASS_IS_ENUM) => {
				Some(eval_no_type_hint(&self.class_name, &self.hint_string, self.usage, defs)?)
			}
			VariantType::OBJECT => {
				Some(eval_no_type_hint(&self.class_name, &self.hint_string, self.usage, defs)?)
			}
			VariantType::ARRAY => {
				let array =
					if self.hint == PropertyHint::ARRAY_TYPE {
						JArray::new(eval_no_type_hint(&self.class_name, &self.hint_string, self.usage, defs)?)
					} else {
						JArray::untyped()
					}.into();

				Some(array)
			}
			_ => None,
		};

		schema.or_else(|| raw_definition_from_type(self.variant_type).map(Type::Definition))
			.ok_or_else(|| anyhow!("Unsupported property type: {:?}", self.variant_type))
	}
}

fn eval_no_type_hint(
	class_name: &StringName,
	hint_string: &str,
	usage: PropertyUsageFlags,
	defs: &mut BTreeMap<String, Definition>,
) -> Result<Type> {
	if usage.is_set(PropertyUsageFlags::CLASS_IS_ENUM) {
		let (enum_def, enum_name) = JEnum::from_enum_path(class_name)?;
		let jref = JRef::new(enum_name);
		defs.insert(jref.name.clone(), enum_def.into());
		return Ok(jref.into());
	}

	if !class_name.is_empty() {
		let class_from_name = ClassSource::from_class_name(class_name.clone())
			.and_then(|source| JClass::generate(source, defs))
			.map(|class| {
				let jref = class.source.to_reference();
				defs.insert(jref.name.clone(), class.into());
				jref.into()
			});

		if let Ok(class) = class_from_name {
			return Ok(class);
		}
	}

	if hint_string.is_empty() {
		return Ok(json_type_of::<Null>());
	}

	if let Some(ty) = VariantDefinition::try_from_name(hint_string) {
		return Ok(ty.into());
	}

	if let Some(ty) = raw_definition_from_name(hint_string) {
		return Ok(ty.into());
	}

	let class_from_hint = ClassSource::from_class_name(hint_string)
		.and_then(|source| JClass::generate(source, defs))
		.map(|class| {
			let jref = class.source.to_reference();
			defs.insert(jref.name.clone(), class.into());
			jref.into()
		});

	if let Ok(class) = class_from_hint {
		return Ok(class);
	}

	let (enum_def, enum_name) = JEnum::from_enum_path(hint_string)?;

	let jref = JRef::new(enum_name);
	defs.insert(jref.name.clone(), enum_def.into());
	Ok(jref.into())
}