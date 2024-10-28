use super::*;
use jsonschema::Validator;

#[derive(GodotClass)]
#[class(no_init, base = RefCounted)]
pub struct GodotSchema {
	pub class: RootSchema,
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
		let result = ClassSource::from_class_name(class_name).and_then(Self::generate);

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
		let result = Self::generate(source);
		
		match result {
			Ok(schema) => Gd::from_object(schema).to_variant(),
			Err(err) => format!("{err:?}").to_variant(),
		}
	}

	/// Instantiates the class defined by this schema from JSON input containing the properties of the class.
	///
	/// Notes:
	/// - The JSON input must be valid according to the schema.
	/// - The JSON input must contain all properties defined in the schema (i.e. the schema's "required" array has all of your class's properties).
	/// - The JSON input must not contain any additional properties (i.e. the schema's "additionalProperties" key is set to false).
	///
	/// # Returns
	/// - The instantiated class, if successful.
	/// - Otherwise, a `String` containing the error message.
	#[func]
	pub fn instantiate(&self, input_json: String) -> Variant {
		let try_fn = || {
			let value = serde_json::from_str(&input_json)?;
			let result = self.validator.validate(&value);

			match result {
				Ok(()) => {
					drop(result);

					if let Value::Object(props) = value {
						self.class.instantiate(&props)
					} else {
						bail!("Expected JSON input to be an object.\nGot `{value:?}`")
					}
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

	/// Returns the JSON schema response format for this class in OpenAI format.
	/// 	
	/// This is useful for calling structured outputs with an LLM using a class-specific schema.
	#[func]
	pub fn open_ai_response_format(&self) -> Variant {
		let name = self.class.base.source.definition_name();
		let schema = &self.class;

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
	pub fn generate(source: ClassSource) -> Result<Self> {
		let class = RootSchema::generate(source)?;

		let json = class.to_json_pretty()?;
		let json_value = serde_json::from_str(&json)?;
		let validator = jsonschema::draft202012::new(&json_value)?;

		Ok(Self {
			class,
			json: json.into(),
			validator,
		})
	}
}