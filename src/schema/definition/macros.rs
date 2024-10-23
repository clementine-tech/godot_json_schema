macro_rules! primitive_definitions {
    ($Source: ident: [$($Alias: ty),* $(,)?]) => {
	    $(
	        impl GetDefinition for $Alias {
				fn get_definition() -> crate::Definition { crate::Definition::$Source() }
			}
	    )*
    };
}

macro_rules! object_definitions {
    ($($Object: ident { 
	    $( $Field: ident : $Type: ty ),* $(,)? 
    })*) => {
	    $(
		    impl crate::GetDefinition for $Object {
				fn get_definition() -> crate::Definition {
					crate::VariantDefinition::$Object.into()
				}
			}
		    
		    impl crate::VariantSourceDefinition for $Object {
			    fn source_definition() -> crate::Definition {
				    {
					    #[allow(unused)]
					    mod __syntax_highlighter {
						    use crate::*;
						    
						    struct Ty {
							    $( $Field: $Type, )*
						    }
					    }
				    }
					
					crate::Builder::object()
				        $( .property(stringify!($Field), crate::definition_of::<$Type>()) )*
						.done()
						.into()
			    }
		    }
	    
	        impl crate::FromJson for $Object {
				fn try_from_json(json: &serde_json::Value) -> Result<Self> {
					let serde_json::Value::Object(properties) = json 
					else { bail!("Expected JSON value to be of type \"object\".\nGot: {json:?}") };
					
					Ok(Self {
						$( $Field: try_value_at_key(stringify!($Field), properties)?, )*
					})
				}
			}
	    )*
    };
}

macro_rules! tuple_definitions {
    ($($T: ident),*) => {
	    impl<$($T: crate::GetDefinition,)*> crate::GetDefinition for ($($T,)*) {
		    fn get_definition() -> crate::Definition {
			    crate::JTuple::new([$(crate::definition_of::<$T>()),*]).into()
		    }
	    }
	    
	    #[allow(unused_assignments)]
	    impl<$($T: crate::FromJson + 'static,)*> FromJson for ($($T,)*) {
			fn try_from_json(json: &Value) -> Result<Self> {
				let serde_json::Value::Array(vec) = json
				else { bail!("Expected JSON value to be of type \"array\".\nGot: {json:?}") };
				
				let len: usize = const {
					let mut count = 0;
					$(
						{
							let _ = std::any::TypeId::of::<$T>;
							count += 1;
						}
					)*
					
					count
				};
				
				if vec.len() != len {
					bail!("Expected JSON array to have {len} elements.\nGot: {:?}", vec.len());
				}
				
				let mut idx = 0;
				Ok(
					($(
						{
							let val = <$T as crate::FromJson>::try_from_json(&vec[idx])?;
							idx += 1;
							val
						}
					,)*)
				)
			}
		}
    };
}

macro_rules! packed_array_definitions {
    ($( $Name: ident: [$T: ty] )*) => {
	    $(
	        impl GetDefinition for $Name {
				fn get_definition() -> crate::Definition { 
					crate::VariantDefinition::$Name.into()
				}
			}
	        
	        impl crate::VariantSourceDefinition for $Name {
			    fn source_definition() -> crate::Definition {
				    crate::definition_of::<godot::prelude::Array<$T>>()
			    }
		    }
	    
	        impl FromJson for $Name {
				fn try_from_json(json: &Value) -> Result<Self> {
					let Value::Array(vec) = json
					else { bail!("Expected JSON value to be of type \"array\".\nGot: {json:?}") };
			
					vec.iter()
						.map(|val| <$T>::try_from_json(val))
						.try_collect()
				}
			}
	    )*
    };
}

macro_rules! impl_json_convert {
    ([$($T: ty),* $(,)?] $Pattern: pat => $Convert: expr) => {
	    $(
	        #[allow(clippy::useless_conversion)]
	        #[allow(clippy::unnecessary_fallible_conversions)]
	        #[allow(clippy::unnecessary_cast)]
	        impl crate::FromJson for $T {
		        fn try_from_json(json: &serde_json::Value) -> Result<Self> {
			        if let $Pattern = json {
				        Ok($Convert)
			        } else {
				        let pat_str = stringify!($Pattern);
				        let json_str = json.to_string();
				        bail!("Expected JSON value to match pattern `{pat_str}`.\nGot:\n\t{json_str}.");
			        }
		        }
	        }
	    )*
    };
}

macro_rules! variant_definitions {
    ($vis: vis enum $E: ident { $( $T: ident = $P: path ),* $(,)? }) => {
	    #[repr(i32)]
		#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord)]
		$vis enum $E {
		    $( $T = $P.ord, )*
	    }
	    
	    impl $E {
		    pub const fn name(&self) -> &'static str {
			    match self {
				    $( $E::$T => stringify!($T), )*
			    }
		    }
		    
			pub fn get_definition(&self) -> Definition {
				match self {
					$( $E::$T => definition_of::<$T>(), )*
				}
			}
		    
		    pub fn var_from_json(&self, json: &Value) -> Result<Variant> {
			    match self {
				    $( $E::$T => <$T as crate::FromJson>::try_from_json(json).map(|v| v.to_variant()), )*
			    }
		    }
		    
		    pub const fn variant_type(&self) -> VariantType {
			    match self {
				    $( $E::$T => $P, )*
			    }
		    }
		}
    };
}

pub(crate) use {
	object_definitions, 
	primitive_definitions, 
	tuple_definitions, 
	packed_array_definitions, 
	variant_definitions, 
	impl_json_convert,
};