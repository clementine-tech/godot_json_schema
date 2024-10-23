use jsonschema::Validator;
use super::*;

pub struct CachedClass {
	pub class: RootSchema,
	pub json: String,
	pub validator: Validator,
}

impl CachedClass {
	pub fn generate(source: ClassSource) -> Result<Self> {
		let class = RootSchema::generate(source)?;

		let json = class.to_json_pretty()?;
		let json_value = serde_json::from_str(&json)?;
		let validator = jsonschema::draft202012::new(&json_value)?;

		Ok(Self {
			class,
			json,
			validator,
		})
	}

	pub fn validate_json_input(&self, input: String) -> Result<Map<String, Value>> {
		let value = serde_json::from_str(&input)?;
		let result = self.validator.validate(&value);

		match result {
			Ok(()) => {
				drop(result);

				if let Value::Object(properties) = value {
					Ok(properties)
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
	}
}
