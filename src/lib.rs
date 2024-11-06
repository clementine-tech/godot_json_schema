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
	#[var] pub schemas: Array<Gd<GodotSchema>>,
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
		let variant = GodotSchema::from_class_name(class_name.clone());
		
		if let Ok(schema) = variant.try_to::<Gd<GodotSchema>>() {
			self.schemas.push(&schema);
		}
		
		variant
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
			self.schemas.push(&schema);
		}

		variant
	}
	
	/// See [`GodotSchema::from_type_info()`]
	/// 
	/// # Returns
	/// - The `GodotSchema` object containing the type's schema, if successful.
	/// - Otherwise a `String` containing the error message.
	#[func]
	pub fn generate_type_info_schema(
		&mut self,
		variant_type: VariantType,
		class_name: StringName,
		hint: PropertyHint,
		hint_string: String,
		usage: PropertyUsageFlags,
	) -> Variant {
		let variant = GodotSchema::from_type_info(
			variant_type,
			class_name,
			hint,
			hint_string,
			usage,
		);
		
		if let Ok(schema) = variant.try_to::<Gd<GodotSchema>>() {
			self.schemas.push(&schema);
		}
		
		variant
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
				self.find_class(source).ok_or_else(|| anyhow!("No schema found for class \"{class_name}\"."))
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

		if let Some(schema) = self.find_class(source) {
			schema.to_variant()
		} else {
			"No schema found for class from input script.".to_variant()
		}
	}
}

impl SchemaLibrary {
	pub fn find_class(&self, source: ClassSource) -> Option<Gd<GodotSchema>> {
		self.schemas.iter_shared().find(|schema| {
			let base = &schema.bind().inner.base;
			
			if let Definition::Class(class) = base {
				class.source == source
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
	#[allow(unused_imports)]
	use clm::*;
	
	struct MyExtension;

	#[gdextension]
	unsafe impl ExtensionLibrary for MyExtension {}
}