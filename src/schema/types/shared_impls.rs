macro_rules! impl_add_description {
    ($($T: ty),*) => {
	    $(	    
	        impl $T {
				pub fn add_description(&mut self, description: impl Into<String>) {
					debug_assert!(self.description.is_none());
			
					self.description = Some(description.into());
				}
			}
	    )*
    };
}

macro_rules! impl_to_json {
    ($($T: ty),*) => {
	    $(	    
	        impl $T {
		        pub fn to_json_compact(&self) -> serde_json::Result<String> {
					serde_json::to_string(self)
				}
				
				pub fn to_json_pretty(&self) -> serde_json::Result<String> {
					serde_json::to_string_pretty(self)
				}
			}
	    )*
    };
}

macro_rules! impl_serialize {
    ($($T: ty),*) => {
	    $(
	        impl Serialize for $T {
				fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
					let mut map = serializer.serialize_map(None)?;
			
					if let Some(description) = &self.description {
						map.serialize_entry("description", description)?;
					}
			
					self.serialize_fields(&mut map)?;
					map.end()
				}
			}
	    )*
    };
}

macro_rules! impl_into_type {
    ($($T: ty),*) => {
	    $(
	        impl From<$T> for Type {
				fn from(val: $T) -> Self {
					Self::Definition(val.into())
				}
			}
	    )*
    };
}

macro_rules! all_shared_impls {
    ($($T: ty),*) => {
	    $(	    
	        $crate::schema::shared_impls::impl_add_description!($T);
	        $crate::schema::shared_impls::impl_to_json!($T);
	        $crate::schema::shared_impls::impl_serialize!($T);
	        $crate::schema::shared_impls::impl_into_type!($T);
	    )*
    };
}

pub(crate) use {impl_add_description, impl_to_json, impl_serialize, impl_into_type, all_shared_impls};