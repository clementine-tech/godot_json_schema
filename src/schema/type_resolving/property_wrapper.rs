use super::*;

pub struct PropertyWrapper {
	pub variant_type: VariantType,
	pub class_name: StringName,
	pub property_name: String,
	pub hint: PropertyHint,
	pub hint_string: String,
	pub usage: PropertyUsageFlags,
}

impl TryFrom<Dictionary> for PropertyWrapper {
	type Error = anyhow::Error;

	fn try_from(dict: Dictionary) -> std::result::Result<Self, Self::Error> {
		Ok(PropertyWrapper {
			property_name: try_get(&dict, "name")?,
			variant_type: try_get(&dict, "type")?,
			class_name: try_get(&dict, "class_name")?,
			hint: try_get(&dict, "hint")?,
			hint_string: try_get(&dict, "hint_string")?,
			usage: try_get(&dict, "usage")?,
		})
	}
}

impl PropertyWrapper {
	pub fn eval_type(&self, defs: &mut BTreeMap<String, Definition>) -> Result<(String, Type)> {
		let schema = match self.variant_type {
			VariantType::INT if self.usage.is_set(PropertyUsageFlags::CLASS_IS_ENUM) => {
				let (enum_def, enum_name) = JEnum::from_enum_path(&self.class_name)?;
				
				let jref = JRef::new(enum_name);
				defs.insert(jref.name.clone(), enum_def.into());
				Some(jref.into())
			}
			VariantType::OBJECT => {
				let class = {
					let source = ClassSource::from_class_name(self.class_name.clone())?;
					JClass::generate(source, defs)?
				};
				
				let jref = class.source.to_reference();
				defs.insert(jref.name.clone(), class.into());
				Some(jref.into())
			}
			VariantType::ARRAY => {
				let array =
					if self.hint == PropertyHint::ARRAY_TYPE {
						let ty = type_from_hint_string(self.hint, &self.hint_string, defs)?;
						JArray::new(ty)
					} else {
						JArray::untyped()
					}.into();

				Some(array)
			}
			_ => None,
		};

		let schema = schema
			.or_else(|| raw_variant_definition(self.variant_type).map(Type::Definition))
			.ok_or_else(|| anyhow!("Unsupported property type: {:?}", self.variant_type))?;

		Ok((self.property_name.to_string(), schema))
	}
}

fn type_from_hint_string(hint: PropertyHint, hint_string: &str, defs: &mut BTreeMap<String, Definition>) -> Result<Type> {
	Ok(match hint {
		PropertyHint::ARRAY_TYPE => {
			if hint_string.is_empty() {
				json_type_of::<Null>()
			} else if let Some(ty) = primitive_from_type_name(hint_string) {
				ty.into()
			} else {
				let class = {
					let source = ClassSource::from_class_name(hint_string)?;
					JClass::generate(source, defs)?
				};
				
				let jref = class.source.to_reference();
				defs.insert(jref.name.clone(), class.into());
				jref.into()
			}
		}
		_ => bail!("Unsupported property hint: {:?}", hint),
	})
}

fn primitive_from_type_name(name: &str) -> Option<Definition> {
	Some(match name {
		"int" => definition_of::<i32>(),
		"float" => definition_of::<f32>(),
		"bool" => definition_of::<bool>(),
		"String" => definition_of::<String>(),
		"Vector2" => definition_of::<Vector2>(),
		_ => return None,
	})
}
