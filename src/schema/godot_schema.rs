use super::*;
use jsonschema::Validator;

#[derive(GodotClass)]
#[class(no_init, base = RefCounted)]
pub struct GodotSchema {
	pub inner: RootSchema,
	pub validator: Validator,
	#[var(get)] pub json: GString,
}

#[godot_api]
impl GodotSchema {
	/// Generates a schema for class named `class_name`.
	///
	/// If it is a GDScript class, it must be registered in `ProjectSettings::get_global_class_list()`. 
	///
	/// For a class to be registered, it needs to contain a "`class_name MyName`" statement at the top of the script.
	///
	/// # Returns
	/// - The `GodotSchema` object containing the class's schema, if successful.
	/// - Otherwise a `String` containing the error message.
	#[func]
	pub fn from_class_name(class_name: StringName) -> Variant {
		let result = ClassSource::from_class_name(class_name)
			.and_then(RootSchema::from_class)
			.and_then(Self::new);

		match result {
			Ok(schema) => Gd::from_object(schema).to_variant(),
			Err(err) => format!("{err:?}").to_variant(),
		}
	}

	/// Generates a schema for a GdScript class defined in `script`.
	///
	/// Unlike [`from_class_name()`](Self::from_class_name), 
	/// this method does not require the class to be registered in `ProjectSettings::get_global_class_list()`.
	///
	/// # Returns
	/// - The `GodotSchema` object containing the class's schema, if successful.
	/// - Otherwise a `String` containing the error message.
	#[func]
	pub fn from_class_script(script: Gd<Script>) -> Variant {
		let source = ClassSource::from_script(script);
		let result = RootSchema::from_class(source).and_then(Self::new);

		match result {
			Ok(schema) => Gd::from_object(schema).to_variant(),
			Err(err) => format!("{err:?}").to_variant(),
		}
	}

	/// Generates a schema for a Godot type.
	///
	/// Godot's type info system is a bit convoluted, read each property's documentation for more info.
	///
	/// - `variant_type`: [Variant.Type](https://docs.godotengine.org/en/stable/classes/class_@globalscope.html#enum-globalscope-variant-type)
	/// - `class_name`: not always the class name, can also be the enum name, see the documentation for `hint` and `usage`
	/// - `hint` [PropertyHint](https://docs.godotengine.org/en/stable/classes/class_@globalscope.html#enum-globalscope-propertyhint)
	/// - `hint_string`: see the documentation for `hint`
	/// - `usage` [PropertyUsageFlags](https://docs.godotengine.org/en/stable/classes/class_@globalscope.html#enum-globalscope-propertyusageflags)
	///
	/// # Input Examples
	/// Arguments not mentioned are set to their default values.
	///
	/// - Vector2 => { type: VariantType::VECTOR2 }
	/// - String => { type: VariantType::STRING }
	/// - Enum `Gender` => { type: VariantType::INT, class_name: &"Person.Gender", usage: PropertyUsageFlags::CLASS_IS_ENUM }
	/// - Class `Fact` => { type: VariantType::OBJECT, class_name: "Fact" }
	/// - UntypedArray => { type: VariantType::ARRAY }
	/// - Dictionary => { type: VariantType::DICTIONARY }
	/// - Array<int> => { type: VariantType::ARRAY, hint: PropertyHint::ARRAY_TYPE, hint_string: "int" } 
	/// - Array<Dictionary> => { type: VariantType::ARRAY, hint: PropertyHint::ARRAY_TYPE, hint_string: "Dictionary"  }
	/// - Array<`Gender`> => { type: VariantType::ARRAY, hint: PropertyHint::ARRAY_TYPE, hint_string: "Person.Gender" }
	/// - Array<`Fact`> => { type: VariantType::ARRAY, hint: PropertyHint::ARRAY_TYPE, hint_string: "Fact" }
	#[func]
	pub fn from_type_info(
		variant_type: VariantType,
		class_name: StringName,
		hint: PropertyHint,
		hint_string: String,
		usage: PropertyUsageFlags,
	) -> Variant {
		let info = PropertyTypeInfo {
			variant_type,
			class_name,
			hint,
			hint_string,
			usage,
			property_name: format!("{variant_type:?}"),
		};

		let result = RootSchema::from_type_info(info).and_then(Self::new);

		match result {
			Ok(inner) => Gd::from_object(inner).to_variant(),
			Err(err) => format!("{err:?}").to_variant(),
		}
	}

	/// Generates a schema for an array of this schema's type.
	/// 
	/// # Input
	/// `item_name`: The array's schema will have a definition of this type named `item_name`.
	///
	/// # Returns
	/// - The `GodotSchema` object containing the array's schema, if successful.
	/// - Otherwise a `String` containing the error message.
	#[func]
	pub fn get_array_schema(&self, item_name: String) -> Variant {
		let mut defs = self.inner.defs.clone();
		let self_def = self.inner.base.clone().into_reference(item_name, &mut defs);

		let array = JArray::new(self_def);
		let schema = RootSchema {
			defs,
			base: array.into(),
		};

		match Self::new(schema) {
			Ok(inner) => Gd::from_object(inner).to_variant(),
			Err(err) => format!("{err:?}").to_variant(),
		}
	}

	/// Instantiates the type defined by this schema from JSON input containing the values of the type.
	///
	/// Notes:
	/// - The JSON input must be valid according to the schema.
	/// - The JSON input must contain all fields defined in the schema (i.e. the schema's "required" array has all of your type's properties).
	/// - The JSON input must not contain any additional properties (i.e. the schema's "additionalProperties" key is set to false).
	///
	/// # Returns
	/// - The instantiated type, if successful.
	/// - Otherwise, a `String` containing the error message.
	#[func]
	pub fn instantiate(&self, input_json: String) -> Variant {
		let try_fn = || {
			let value = serde_json::from_str(&input_json)?;
			let result = self.validator.validate(&value);

			match result {
				Ok(()) => {
					drop(result);

					// If we are a wrapper for a non-class type, the actual input is in the "value" property.
					let value =
						if let Value::Object(properties) = &value
							&& properties.len() == 1
							&& let Some(inner) = properties.get("value")
							&& !matches!(self.inner.base, Definition::Class(_) | Definition::Object(_)) {
							inner
						} else {
							&value
						};
					
					self.inner.instantiate(value)
				}
				Err(errors) => {
					let mut msg = String::new();

					for err in errors {
						msg += &format!("{err:?}\n");
					}

					bail!("{msg}")
				}
			}
		};

		match try_fn() {
			Ok(obj) => obj.to_variant(),
			Err(err) => format!("{err}").to_variant(),
		}
	}

	/// Returns the JSON schema response format for this schema in OpenAI format.
	/// 	
	/// This is useful for calling structured outputs with an LLM using a type-specific schema.
	/// 
	/// # Input
	/// `name`: The root name of the schema, must be a valid identifier. (Cannot contain spaces)
	#[func]
	pub fn open_ai_response_format(&self, name: String) -> Variant {
		let schema = &self.inner;

		let result = std::panic::catch_unwind(||
			serde_json::json!({
				"type": "json_schema",
				"json_schema": {
					"name": name,
					"schema": schema,
				},
			}))
			.map_err(|err| anyhow!("{err:?}"))
			.and_then(|value| {
				// In integration tests, return a bigger but more readable JSON.
				#[cfg(feature = "integration_tests")]
				return serde_json::to_string_pretty(&value).map_err(anyhow::Error::from);

				#[cfg(not(feature = "integration_tests"))]
				return serde_json::to_string(&value).map_err(anyhow::Error::from);
			});

		match result {
			Ok(json) => json.to_variant(),
			Err(err) => {
				godot_error!("{err}");
				String::default().to_variant()
			}
		}
	}
}

impl GodotSchema {
	pub fn new(schema: RootSchema) -> Result<Self> {
		let json = schema.to_json_pretty()?;
		let json_value = serde_json::from_str(&json)?;
		let validator = jsonschema::draft202012::new(&json_value)?;

		Ok(Self {
			inner: schema,
			json: json.into(),
			validator,
		})
	}
}