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
	pub schemas: HashMap<ClassSource, Gd<GodotSchema>>,
}

#[godot_api]
impl SchemaLibrary {
	/// Generates a schema for class named `class_name`.
	///
	/// If it is a GDScript class, it must be registered in [`ProjectSettings::get_global_class_list()`]. 
	///
	/// For a class to be registered, it needs to contain a "`class_name MyName`" statement at the top of the script.
	///
	/// # Returns
	/// - The `GodotSchema` object containing the class's schema, if successful.
	/// - Otherwise a `String` containing the error message.
	#[func]
	pub fn generate_named_class_schema(&mut self, class_name: StringName) -> Variant {
		let variant = GodotSchema::from_class_name(class_name);
		
		if let Ok(schema) = variant.try_to::<Gd<GodotSchema>>() {
			let source = schema.bind().class.base.source.clone();
			self.schemas.insert(source, schema.clone());
			schema.to_variant()
		} else {
			variant
		}
	}

	/// Generates a schema for a GdScript class defined in `script`.
	///
	/// Unlike [`Self::generate_named_class_schema()`], 
	/// this method does not require the class to be registered in [`ProjectSettings::get_global_class_list()`].
	///
	/// # Returns
	/// - The `GodotSchema` object containing the class's schema, if successful.
	/// - Otherwise a `String` containing the error message.
	#[func]
	pub fn generate_unnamed_class_schema(&mut self, script: Gd<Script>) -> Variant {
		let variant = GodotSchema::from_class_script(script);

		if let Ok(schema) = variant.try_to::<Gd<GodotSchema>>() {
			let source = schema.bind().class.base.source.clone();
			self.schemas.insert(source, schema.clone());
			schema.to_variant()
		} else {
			variant
		}
	}

	/// Returns the `GodotSchema` object containing the schema of class named `class_name`.
	///
	/// If the schema was generated from a GDScript class that does not have a global name
	/// (i.e. it has a "class_name MyName" statement at the top of the script),
	/// use [`Self::get_unnamed_class_schema()`] instead.
	///
	/// # Returns
	/// - The `GodotSchema` object containing the class's schema, if found.
	/// - Otherwise a `String` containing the error message.
	#[func]
	pub fn get_named_class_schema(&self, class_name: StringName) -> Variant {
		let result = ClassSource::from_class_name(class_name.clone())
			.and_then(|source| {
				self.schemas
					.get(&source)
					.ok_or_else(|| anyhow!("No schema found for class \"{class_name}\"."))
			});
		
		match result {
			Ok(schema) => schema.to_variant(),
			Err(err) => format!("{err:?}").to_variant(),
		}
	}

	/// Returns the `GodotSchema` object containing the schema of class defined in `script`.
	/// 
	/// Unlike [`Self::get_named_class_schema()`], this method does not require the class to be registered in [`ProjectSettings::get_global_class_list()`].
	///
	/// # Returns
	/// - The `GodotSchema` object containing the class's schema, if found.
	/// - Otherwise a `String` containing the error message.
	#[func]
	pub fn get_unnamed_class_schema(&self, script: Gd<Script>) -> Variant {
		let source = ClassSource::from_script(script.clone());

		if let Some(schema) = self.schemas.get(&source) {
			schema.to_variant()
		} else {
			"No schema found for class from input script.".to_variant()
		}
	}
	
	/// Adds a schema to the library.
	/// 
	/// Manually doing this is only necessary if you did not generate the schema using either
	/// [`Self::generate_named_class_schema()`] or [`Self::generate_unnamed_class_schema()`].
	#[func]
	pub fn add_schema(&mut self, schema: Gd<GodotSchema>) {
		let source = schema.bind().class.base.source.clone();
		self.schemas.insert(source, schema);
	}
	
	/// Removes a schema from the library.
	#[func]
	pub fn remove_schema(&mut self, schema: Gd<GodotSchema>) {
		let source = schema.bind().class.base.source.clone();
		self.schemas.remove(&source);
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
	#[allow(unused_imports)]
	use clm::*;
	
	struct MyExtension;

	#[gdextension]
	unsafe impl ExtensionLibrary for MyExtension {}
}
