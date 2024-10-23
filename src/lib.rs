#![feature(let_chains)]
#![allow(non_camel_case_types)]
#![warn(clippy::missing_const_for_fn)]

//! Generate JSON schemas from Godot classes, and instantiate Godot classes from json.
//!
//! # Setup Example (GDScript)
//!
//! Consider the given class:
//! ```js
//! class_name Person
//!
//! var name: String
//! var age: int
//! ```
//!
//! ## Step 1
//! Create a `SchemaLibrary` instance, which will be used to generate and cache your schemas:
//! ```js
//! var library = SchemaLibrary.new()
//! ```
//!
//! ## Step 2
//! Generate a schema for your class.
//!
//! Note 1: If the class is "user-made" (i.e. not part of the engine),
//! the generated schema will only contain the properties of the class and any other GDScript classes that it inherits from. 
//! This means that properties from any ancestor engine classes that you inherit from will not be included in the schema.
//!
//! Note 2: If a property of your class is a Godot class, you do not have to generate a schema for the property's type,
//! this crate will automatically generate all the necessary "dependency" schemas for you.
//! These dependencies are included as definitions in the schema (i.e. included in the "$defs" dictionary).
//!
//! A schema can be generated in two different ways:
//!
//! ### Method 1 - "`generate_named_class_schema(ClassName)`"
//! Requires your class to be registered in `ProjectSettings::get_global_class_list()`.
//!
//! For a class to be registered, it needs to contain a "`class_name MyName`" statement at the top of the script.
//!
//! "`generate_named_class_schema()`" will return "`Nil`"(if it succeeded) or "`String`" as an error message.
//!
//! ```js
//! var result = library.generate_named_class_schema(&"Person")
//!
//! if result is String:
//!     print_err(result)
//! ```
//!
//! ### Method 2 - "`generate_unnamed_class_schema(Script)`"
//! Does not require your class to be registered in `ProjectSettings::get_global_class_list()`.
//!
//! "`generate_unnamed_class_schema`" will return "`Nil`"(if it succeeded) or "`String`" as an error message.
//!
//! ```js
//! var script = preload("res://person.gd")
//! var result = library.generate_unnamed_class_schema(script)
//!
//! if result is String:
//!     print_err(result)
//! ```
//!
//! # Instantiating Godot classes from JSON
//!
//! After setting up your library and ensuring that your necessary schemas are generated,
//! you can instantiate a given class from JSON input containing the properties of the class.
//!
//! Notes:
//! - The JSON input must be valid according to the schema.
//! - The JSON input must contain all properties defined in the schema (i.e. the schema's "required" array has all of your class's properties).
//! - The JSON input must not contain any additional properties (i.e. the schema's "additionalProperties" key is set to false).
//!
//! ```js
//! var person_properties_json = 
//!     """
//!     {
//!         "name": "John Doe",
//!         "age": 43
//!     }
//!     """
//!
//! var result = library.instantiate_named_class(&"Person", person_properties_json)
//!
//! if result is String:
//!     print_err(result)
//! else:
//!     var person: Person = result
//!     assert(person.name == "John Doe")
//!     assert(person.age == 43)
//! ```
//! 
//! Note: If the class does not have a global name (i.e. it has a "class_name MyName" statement at the top of the script),
//! use "`instantiate_unnamed_class(Script, PropertiesJson)`" instead.
//!
//! # Accessing the json representation of a generated schema
//!
//! Use either "`get_named_class_schema(ClassName)`" or "`get_unnamed_class_schema(Script)`" on your `SchemaLibrary` instance.
//!
//! ```js
//! var json: String = library.get_named_class_schema(&"Person")
//!
//! if json.is_empty():
//!     print_err("No schema found for class Person.")
//! else:
//!     print("Generated schema:\n" + json)
//! ```

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
	
	struct MyExtension;

	#[gdextension]
	unsafe impl ExtensionLibrary for MyExtension {}
}