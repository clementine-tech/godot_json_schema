use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ClassId {
	Name(StringName),
	Script(Gd<Script>),
}

impl ClassId {
	pub fn definition_name(&self) -> String {
		match self {
			ClassId::Name(name) => name.to_string(),
			ClassId::Script(script) => script.get_path().to_string(),
		}
	}
}