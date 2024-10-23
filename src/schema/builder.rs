use crate::schema::types::j_enum::JEnum;
use crate::schema::types::object::JObject;
use super::*;

#[derive(Default)]
pub struct Builder<T = ()> {
	inner: T,
}

impl Builder {
	pub fn object() -> Builder<JObject> { Builder::default() }
	pub fn string_enum() -> Builder<JEnum> { Builder::default() }
}

impl<T> Builder<T> {
	pub fn new() -> Self where T: Default { Self::default() }
	
	pub fn done(self) -> T {
		self.inner
	}
}

impl Builder<JObject> {
	pub fn description(self, description: impl Into<String>) -> Self {
		debug_assert!(self.inner.description.is_none());
		
		Self {
			inner: JObject {
				description: Some(description.into()),
				..self.inner
			}
		}
	}
	
	pub fn property(mut self, name: impl Into<String>, ty: impl Into<Type>) -> Self {
		self.inner.add_property(name, ty);
		self
	}
}

impl Builder<JEnum> {
	pub fn description(self, description: impl Into<String>) -> Self {
		debug_assert!(self.inner.description.is_none());
		
		Self {
			inner: JEnum {
				description: Some(description.into()),
				..self.inner
			}
		}
	}
	
	pub fn variant(mut self, name: impl Into<String>, value: impl Into<i64>) -> Self {
		self.inner.add_variant(name, value);
		self
	}
}