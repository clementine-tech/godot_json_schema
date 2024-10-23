use super::*;

pub use builder::*;
pub use types::*;
pub use type_resolving::*;

pub mod builder;
pub mod types;
pub mod type_resolving;
pub mod definition;

trait SerializeFields {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error>;
}

fn try_get<T: FromGodot>(dict: &Dictionary, key: &str) -> Result<T> {
	dict.get(key)
		.ok_or_else(|| anyhow!("Expected key `name` in property dictionary"))?
		.try_to()
		.map_err(|err| anyhow!("{err:?}"))
}