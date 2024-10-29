use super::*;

variant_definitions! {
	pub enum VariantDefinition {
		Vector2 = VariantType::VECTOR2,
		Vector2i = VariantType::VECTOR2I,
		Rect2 = VariantType::RECT2,
		Rect2i = VariantType::RECT2I,
		Vector3 = VariantType::VECTOR3,
		Vector3i = VariantType::VECTOR3I,
		Transform2D = VariantType::TRANSFORM2D,
		Vector4 = VariantType::VECTOR4,
		Vector4i = VariantType::VECTOR4I,
		Plane = VariantType::PLANE,
		Quaternion = VariantType::QUATERNION,
		Aabb = VariantType::AABB,
		Basis = VariantType::BASIS,
		Transform3D = VariantType::TRANSFORM3D,
		Projection = VariantType::PROJECTION,
		Color = VariantType::COLOR,
		Rid = VariantType::RID,
		PackedByteArray = VariantType::PACKED_BYTE_ARRAY,
		PackedInt32Array = VariantType::PACKED_INT32_ARRAY,
		PackedInt64Array = VariantType::PACKED_INT64_ARRAY,
		PackedFloat32Array = VariantType::PACKED_FLOAT32_ARRAY,
		PackedFloat64Array = VariantType::PACKED_FLOAT64_ARRAY,
		PackedStringArray = VariantType::PACKED_STRING_ARRAY,
		PackedVector2Array = VariantType::PACKED_VECTOR2_ARRAY,
		PackedVector3Array = VariantType::PACKED_VECTOR3_ARRAY,
		PackedColorArray = VariantType::PACKED_COLOR_ARRAY,
		PackedVector4Array = VariantType::PACKED_VECTOR4_ARRAY,
	}
}

impl_to_json!(VariantDefinition);
impl_into_type!(VariantDefinition);

impl VariantDefinition {
	pub const fn description(&self) -> Option<&String> {
		None
	}
	
	/// Don't use, this is for compatibility with the enum `Definition`.
	pub fn add_description(&mut self, _: impl Into<String>) {
		godot_warn!("`VariantDefinition::add_description` is not allowed.");
	}

	pub fn insert_variant_definitions(&self, fill_me: &mut Vec<VariantDefinition>) {
		fill_me.push(*self);

		match self {
			| VariantDefinition::Rect2
			| VariantDefinition::Transform2D
			| VariantDefinition::PackedVector2Array => fill_me.push(VariantDefinition::Vector2),

			| VariantDefinition::Plane
			| VariantDefinition::Aabb
			| VariantDefinition::Basis
			| VariantDefinition::PackedVector3Array => fill_me.push(VariantDefinition::Vector3),

			| VariantDefinition::Projection
			| VariantDefinition::PackedVector4Array => fill_me.push(VariantDefinition::Vector4),

			VariantDefinition::Rect2i => fill_me.push(VariantDefinition::Vector2i),
			VariantDefinition::PackedColorArray => fill_me.push(VariantDefinition::Color),
			VariantDefinition::Transform3D => fill_me.extend([
				VariantDefinition::Vector3,
				VariantDefinition::Basis,
			]),
			_ => {}
		}
	}

	pub fn source_definition(&self) -> Definition {
		match self {
			VariantDefinition::Vector2 => Vector2::source_definition(),
			VariantDefinition::Vector2i => Vector2i::source_definition(),
			VariantDefinition::Rect2 => Rect2::source_definition(),
			VariantDefinition::Rect2i => Rect2i::source_definition(),
			VariantDefinition::Vector3 => Vector3::source_definition(),
			VariantDefinition::Vector3i => Vector3i::source_definition(),
			VariantDefinition::Transform2D => Transform2D::source_definition(),
			VariantDefinition::Vector4 => Vector4::source_definition(),
			VariantDefinition::Vector4i => Vector4i::source_definition(),
			VariantDefinition::Plane => Plane::source_definition(),
			VariantDefinition::Quaternion => Quaternion::source_definition(),
			VariantDefinition::Aabb => Aabb::source_definition(),
			VariantDefinition::Basis => Basis::source_definition(),
			VariantDefinition::Transform3D => Transform3D::source_definition(),
			VariantDefinition::Projection => Projection::source_definition(),
			VariantDefinition::Color => Color::source_definition(),
			VariantDefinition::PackedByteArray => PackedByteArray::source_definition(),
			VariantDefinition::PackedInt32Array => PackedInt32Array::source_definition(),
			VariantDefinition::PackedInt64Array => PackedInt64Array::source_definition(),
			VariantDefinition::PackedFloat32Array => PackedFloat32Array::source_definition(),
			VariantDefinition::PackedFloat64Array => PackedFloat64Array::source_definition(),
			VariantDefinition::PackedStringArray => PackedStringArray::source_definition(),
			VariantDefinition::PackedVector2Array => PackedVector2Array::source_definition(),
			VariantDefinition::PackedVector3Array => PackedVector3Array::source_definition(),
			VariantDefinition::PackedColorArray => PackedColorArray::source_definition(),
			VariantDefinition::PackedVector4Array => PackedVector4Array::source_definition(),
			VariantDefinition::Rid => self.get_definition(),
		}
	}
}

pub trait VariantSourceDefinition {
	fn source_definition() -> Definition;
}

impl Serialize for VariantDefinition {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		let mut map = serializer.serialize_map(Some(1))?;
		self.serialize_fields(&mut map)?;
		map.end()
	}
}

impl SerializeFields for VariantDefinition {
	fn serialize_fields<M: SerializeMap>(&self, map: &mut M) -> Result<(), M::Error> {
		map.serialize_entry("$ref", &format!("#/$defs/{}", self.name()))
	}
}

impl TryFrom<VariantType> for VariantDefinition {
	type Error = ();

	fn try_from(value: VariantType) -> std::result::Result<Self, Self::Error> {
		match value {
			VariantType::VECTOR2 => Ok(VariantDefinition::Vector2),
			VariantType::VECTOR2I => Ok(VariantDefinition::Vector2i),
			VariantType::RECT2 => Ok(VariantDefinition::Rect2),
			VariantType::RECT2I => Ok(VariantDefinition::Rect2i),
			VariantType::VECTOR3 => Ok(VariantDefinition::Vector3),
			VariantType::VECTOR3I => Ok(VariantDefinition::Vector3i),
			VariantType::TRANSFORM2D => Ok(VariantDefinition::Transform2D),
			VariantType::VECTOR4 => Ok(VariantDefinition::Vector4),
			VariantType::VECTOR4I => Ok(VariantDefinition::Vector4i),
			VariantType::PLANE => Ok(VariantDefinition::Plane),
			VariantType::QUATERNION => Ok(VariantDefinition::Quaternion),
			VariantType::AABB => Ok(VariantDefinition::Aabb),
			VariantType::BASIS => Ok(VariantDefinition::Basis),
			VariantType::TRANSFORM3D => Ok(VariantDefinition::Transform3D),
			VariantType::PROJECTION => Ok(VariantDefinition::Projection),
			VariantType::COLOR => Ok(VariantDefinition::Color),
			VariantType::PACKED_BYTE_ARRAY => Ok(VariantDefinition::PackedByteArray),
			VariantType::PACKED_INT32_ARRAY => Ok(VariantDefinition::PackedInt32Array),
			VariantType::PACKED_INT64_ARRAY => Ok(VariantDefinition::PackedInt64Array),
			VariantType::PACKED_FLOAT32_ARRAY => Ok(VariantDefinition::PackedFloat32Array),
			VariantType::PACKED_FLOAT64_ARRAY => Ok(VariantDefinition::PackedFloat64Array),
			VariantType::PACKED_STRING_ARRAY => Ok(VariantDefinition::PackedStringArray),
			VariantType::PACKED_VECTOR2_ARRAY => Ok(VariantDefinition::PackedVector2Array),
			VariantType::PACKED_VECTOR3_ARRAY => Ok(VariantDefinition::PackedVector3Array),
			VariantType::PACKED_COLOR_ARRAY => Ok(VariantDefinition::PackedColorArray),
			VariantType::PACKED_VECTOR4_ARRAY => Ok(VariantDefinition::PackedVector4Array),
			VariantType::RID => Ok(VariantDefinition::Rid),
			_ => Err(()),
		}
	}
}