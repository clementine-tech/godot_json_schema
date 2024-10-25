#![feature(let_chains)]
#![allow(non_camel_case_types)]
#![warn(clippy::missing_const_for_fn)]

#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod schema;

/// Generates and caches JSON schemas generated from Godot classes.
#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct SchemaLibrary {
	pub schemas: HashMap<ClassId, CachedClass>,
}

#[godot_api]
impl SchemaLibrary {
	/// Generates a schema for class named `class_name`.
	///
	/// If it is a GDScript class, it must be registered in `ProjectSettings::get_global_class_list()`. 
	///
	/// For a class to be registered, it needs to contain a "`class_name MyName`" statement at the top of the script.
	///
	/// # Returns
	/// - `Variant::nil()` if the schema was successfully generated.
	/// - Otherwise a `String` containing the error message.
	#[func]
	pub fn generate_named_class_schema(&mut self, class_name: StringName) -> Variant {
		self.generate_schema(ClassSource::from_class_name(class_name.clone()))
	}

	/// Generates a schema for a GdScript class defined in `script`.
	///
	/// Unlike `generate_named_class_schema`, this method does not require the class to be registered in `ProjectSettings::get_global_class_list()`.
	///
	/// # Returns
	/// - `Variant::nil()` if the schema was successfully generated.
	/// - Otherwise, a `String` containing the error message.
	#[func]
	pub fn generate_unnamed_class_schema(&mut self, script: Gd<Script>) -> Variant {
		self.generate_schema(Ok(ClassSource::from_script(script)))
	}

	/// Returns the JSON representation of the schema for a class named `class_name`.
	///
	/// If the schema was generated from a GDScript class that does not have a global name
	/// (i.e. it has a "class_name MyName" statement at the top of the script),
	/// use `get_unnamed_class_schema` instead.
	///
	/// # Returns
	/// - A `String` containing the JSON representation of the schema, if found.
	/// - Otherwise, an empty `String`.
	#[func]
	pub fn get_named_class_schema(&self, class_name: StringName) -> Variant {
		self.get_schema(ClassId::Name(class_name))
	}

	/// Returns the JSON representation of the schema for a class defined in `script`.
	/// 
	/// Unlike `get_named_class_schema`, this method does not require the class to be registered in `ProjectSettings::get_global_class_list()`.
	///
	/// # Returns
	/// - A `String` containing the JSON representation of the schema, if found.
	/// - Otherwise, an empty `String`.
	#[func]
	pub fn get_unnamed_class_schema(&self, script: Gd<Script>) -> Variant {
		self.get_schema(ClassId::Script(script))
	}

	/// Instantiates the class named `class_name` from JSON input containing the properties of the class.
	///
	/// If the class's schema was generated from a GDScript class that does not have a global name
	/// (i.e. it has a "class_name MyName" statement at the top of the script),
	/// use `instantiate_unnamed_class` instead.
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
	pub fn instantiate_named_class(&self, class_name: StringName, properties_json: String) -> Variant {
		self.instantiate_class(ClassId::Name(class_name), properties_json)
	}

	/// Instantiates the class defined in `script` from JSON input containing the properties of the class.
	/// 
	/// Unlike `instantiate_named_class`, this method does not require the class to be registered in `ProjectSettings::get_global_class_list()`.
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
	pub fn instantiate_unnamed_class(&self, script: Gd<Script>, properties_json: String) -> Variant {
		self.instantiate_class(ClassId::Script(script), properties_json)
	}

	/// Constructs a JSON schema response format for a class in OpenAI format.	
	/// This is useful for calling structured outputs with an LLM using a class-specific schema.
	#[func]
	pub fn construct_response_format(&self, schema: String, class_name: String) -> String {
		format!("{{ \"type\": \"json_schema\", \"json_schema\": {{ \"name\": \"{class_name}\", \"schema\": {schema} }} }}")
	}

}

impl SchemaLibrary {
	pub fn generate_schema(&mut self, source_result: Result<ClassSource>) -> Variant {
		let result = source_result
			.and_then(CachedClass::generate);

		match result {
			Ok(cache) => {
				self.schemas.insert(cache.class.base.source.id(), cache);
				Variant::nil()
			}
			Err(err) => {
				format!("{err:?}").to_variant()
			}
		}
	}

	pub fn get_schema(&self, class_id: ClassId) -> Variant {
		let option = self
			.schemas
			.get(&class_id)
			.or_else(|| {
				// It's possible that the class has a global name, yet the user is providing a script.
				if let ClassId::Script(script) = &class_id {
					self.find_class_by_script(script)
				} else {
					None
				}
			});
		
		match option {
			Some(cache) => cache.json.to_variant(),
			None => GString::new().to_variant(),
		}
	}

	pub fn instantiate_class(&self, class_id: ClassId, properties_json: String) -> Variant {
		let Some(cache) = self
			.schemas
			.get(&class_id)
			.or_else(|| {
				// It's possible that the class has a global name, yet the user is providing a script.
				if let ClassId::Script(script) = &class_id {
					self.find_class_by_script(script)
				} else {
					None
				}
			})
		else {
			return format!("No schema found for class with id `{class_id:?}`.\n\n\
							Help: You can generate schemas with `SchemaLibrary::generate_schema_from_class`").to_variant()
		};

		cache.validate_json_input(properties_json)
			.and_then(|props| cache.class.instantiate(&props))
			.map(|obj| obj.to_variant())
			.unwrap_or_else(|err| format!("{err}").to_variant())
	}

	fn find_class_by_script(&self, class_script: &Gd<Script>) -> Option<&CachedClass> {
		self.schemas.values().find(|schema| {
			if let ClassSource::Script { script, .. } = &schema.class.base.source {
				script == class_script
			} else {
				false
			}
		})
	}
}

use internal_prelude::*;

mod internal_prelude {
	pub(crate) use crate::schema::*;
	pub(crate) use anyhow::{anyhow, bail, Result};
	pub(crate) use declarative_type_state::delegated_enum;
	pub(crate) use godot::classes::{ClassDb, ProjectSettings, ResourceLoader, Script};
	pub(crate) use godot::global::{PropertyHint, PropertyUsageFlags};
	pub(crate) use godot::prelude::*;
	pub(crate) use itertools::Itertools;
	pub(crate) use serde::ser::SerializeMap;
	pub(crate) use serde::{Serialize, Serializer};
	pub(crate) use serde_json::{Map, Value};
	pub(crate) use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
	pub(crate) use std::hash::Hash;
}

#[cfg(feature = "integration_tests")]
mod gd_ext_lib {
	use super::*;
	use clm::*;
	
	struct MyExtension;

	#[gdextension]
	unsafe impl ExtensionLibrary for MyExtension {}

	#[derive(GodotClass)]
	#[class(base=Node)]
	struct DummyPlayer {
	}

	use godot::classes::INode;

	#[godot_api]
	impl INode for DummyPlayer {
		fn init(base: Base<Node>) -> Self {			
			Self {}
		}
	}
}
