use super::*;

#[derive(Clone, Debug, Default)]
pub struct JEnum {
	pub description: Option<String>,
	pub variants: BTreeMap<String, i64>,
}

impl JEnum {
	pub fn new(variants: impl Iterator<Item = (impl Into<String>, impl Into<i64>)>) -> Self {
		Self {
			description: None,
			variants: variants
				.map(|(k, v)| (k.into(), v.into()))
				.collect(),
		}
	}
	
	pub fn from_enum_path(enum_path: impl Into<String>) -> Result<(Self, String)> {
		let enum_path = enum_path.into();
		
		let split = enum_path.split(".").collect::<Vec<_>>();
		
		if split.is_empty() {
			bail!("Cannot fetch variant list from enum {enum_path}.\nGlobal enums are not supported.");
		}
		
		if split.len() != 2 {
			bail!("Expected split by '.' to have exactly 2 parts: `ClassName.EnumName` .\nGot: {split:?}");
		}
		
		let class_source = ClassSource::from_class_name(split[0])?;
		let enum_name = split[1];
		let def = Self::from_class_source(&class_source, enum_name)?;
		Ok((def, enum_name.to_owned()))
	}
	
	pub fn from_class_source(source: &ClassSource, enum_name: impl Into<StringName>) -> Result<Self> {
		match source {
			| ClassSource::ScriptNamed(script, _)
			| ClassSource::ScriptUnnamed(script) => Self::from_gdscript_class(script.clone(), enum_name),
			
			ClassSource::Engine(class_name) => Self::from_engine_class(class_name.clone(), enum_name),
		}
	}

	pub fn from_gdscript_class(mut script: Gd<Script>, enum_name: impl Into<StringName>) -> Result<Self> {
		let name = enum_name.into();
		let constants = script.get_script_constant_map();

		let variants = try_get::<Dictionary>(&constants, &name.to_string())?
			.iter_shared()
			.map(|(key_var, value_var)| {
				let key = key_var.try_to::<String>().map_err(|e| anyhow!("{e:?}"))?;
				let value = value_var.try_to::<i64>().map_err(|e| anyhow!("{e:?}"))?;

				Result::<(String, i64)>::Ok((key, value))
			}).try_collect()?;

		Ok(Self {
			description: None,
			variants,
		})
	}

	pub fn from_engine_class(class_name: impl Into<StringName>, enum_name: impl Into<StringName>) -> Result<Self> {
		let class_name = class_name.into();
		let enum_name = enum_name.into();

		let class_db = ClassDb::singleton();

		let variant_names = class_db
			.class_get_enum_constants_ex(class_name.clone(), enum_name.clone())
			.no_inheritance(false)
			.done();

		let variants = variant_names
			.as_slice()
			.iter()
			.map(|name| {
				let value = class_db.class_get_integer_constant(class_name.clone(), name.into());
				(name.to_string(), value)
			}).collect::<BTreeMap<_, _>>();

		if variants.len() > 1 {
			Ok(Self {
				description: None,
				variants,
			})
		} else {
			bail!("Expected enum \"{enum_name}\" to have at least 2 variants.\nGot: {}", variants.len())
		}
	}

	pub fn add_variant(&mut self, variant: impl Into<String>, value: impl Into<i64>) {
		self.variants.insert(variant.into(), value.into());
	}
}

impl SerializeFields for JEnum {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("type", "string")?;
		map.serialize_entry("enum", &self.variants.keys().collect::<Vec<_>>())
	}
}

all_shared_impls!(JEnum);