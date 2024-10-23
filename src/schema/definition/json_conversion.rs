use super::*;
use godot::meta::ArrayElement;
use crate::schema::utils::raw_variant_from_json;

primitive_definitions!(null: [Null, ()]);
primitive_definitions!(boolean: [bool]);
primitive_definitions!(integer: [Integer, i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, Rid]);
primitive_definitions!(number: [Number, f32, f64]);
primitive_definitions!(string: [JString, String, GString, StringName, NodePath]);
primitive_definitions!(untyped_array: [VariantArray, Vec<Variant>]);
primitive_definitions!(dictionary: [Dictionary]);

impl_json_convert! {
	[Null, ()] 
	Value::Null => Self::default()
}

impl_json_convert! { 
	[bool] 
	Value::Bool(val) => *val
}

impl_json_convert! {
	[i8, i16, i32, i64, i128, isize]
	Value::Number(number) => number
		.as_i64()
		.ok_or_else(|| anyhow!("Expected integer, got float."))
		.and_then(|val| val.try_into().map_err(|err| anyhow!("{err}")))?
}

impl_json_convert! {
	[u8, u16, u32, u64, u128, usize]
	Value::Number(number) => number
		.as_u64()
		.ok_or_else(|| anyhow!("Expected positive integer, got: {number}."))
		.and_then(|val| val.try_into().map_err(|err| anyhow!("{err}")))?
}

impl_json_convert! {
	[Rid]
	Value::Number(number) => number
		.as_u64()
		.ok_or_else(|| anyhow!("Expected positive integer, got: {number}."))
		.map(Rid::new)?
}

impl_json_convert! {
	[f32, f64]
	Value::Number(number) => number
		.as_f64()
		.ok_or_else(|| anyhow!("Expected float, got integer."))
		.map(|val| val as Self)?
}

impl_json_convert! {
	[String, GString, StringName, NodePath]
	Value::String(string) => string.into()
}

impl_json_convert! {
	[Dictionary]
	Value::Object(properties) =>
		properties
			.iter()
			.map(|(key, value)| {
				let pair = (key.to_owned(), raw_variant_from_json(value)?);
				Result::<(String, Variant)>::Ok(pair)
			})
			.try_collect()?
}

object_definitions!(
	Vector2  { x: f32, y: f32 }
	Vector2i { x: i32, y: i32 }
	Vector3  { x: f32, y: f32, z: f32 }
	Vector3i { x: i32, y: i32, z: i32 }
	Vector4  { x: f32, y: f32, z: f32, w: f32 }
	Vector4i { x: i32, y: i32, z: i32, w: i32 }
	Rect2 { position: Vector2, size: Vector2 }
	Rect2i { position: Vector2i, size: Vector2i }
	Transform2D { a: Vector2, b: Vector2, origin: Vector2 }
	Transform3D { basis: Basis, origin: Vector3 }
	Plane { normal: Vector3, d: real }
	Quaternion { x: real, y: real, z: real, w: real }
	Aabb { position: Vector3, size: Vector3 }
	Basis { rows: [Vector3; 3] }
	Projection { cols: [Vector4; 4] }
	Color { r: f32, g: f32, b: f32, a: f32 }
);

// Fixed Rust arrays == Tuples in Json 
impl<T: GetDefinition, const N: usize> GetDefinition for [T; N] {
	fn get_definition() -> Definition {
		let def = definition_of::<T>();
		let mut def_list = Vec::with_capacity(N);

		for _ in 0..N {
			def_list.push(def.clone());
		}

		JTuple::new(def_list).into()
	}
}

impl<T: FromJson, const N: usize> FromJson for [T; N] {
	fn try_from_json(json: &Value) -> Result<Self> {
		let Value::Array(vec) = json
		else { bail!("Expected JSON value to be of type \"array\".\nGot: {json:?}") };

		if vec.len() != N {
			bail!("Expected JSON array to have {N} elements.\nGot: {vec:?}");
		}

		let converted_values = vec
			.iter()
			.map(|val| T::try_from_json(val))
			.try_collect::<_, Vec<_>, _>()?;

		// SAFETY: We checked the length of the array above.
		Ok(unsafe { Self::try_from(converted_values).unwrap_unchecked() })
	}
}

impl<K: Into<String>, V: GetDefinition> GetDefinition for HashMap<K, V> {
	fn get_definition() -> Definition { Definition::dictionary() }
}

impl<K: Eq + Hash, V: FromJson> FromJson for HashMap<K, V> where for<'a> K: From<&'a str> {
	fn try_from_json(json: &Value) -> Result<Self> {
		let Value::Object(properties) = json
		else { bail!("Expected JSON value to be of type \"object\".\nGot: {json:?}") };

		properties
			.iter()
			.map(|(key, value)| Ok((K::from(key), V::try_from_json(value)?)))
			.try_collect()
	}
}

impl<K: Into<String>, V: GetDefinition> GetDefinition for BTreeMap<K, V> {
	fn get_definition() -> Definition { Definition::dictionary() }
}

impl<K: Ord, V: FromJson> FromJson for BTreeMap<K, V> where for<'a> K: From<&'a str> {
	fn try_from_json(json: &Value) -> Result<Self> {
		let Value::Object(properties) = json
		else { bail!("Expected JSON value to be of type \"object\".\nGot: {json:?}") };

		properties
			.iter()
			.map(|(key, value)| Ok((K::from(key), V::try_from_json(value)?)))
			.try_collect()
	}
}

impl<T: GetDefinition> GetDefinition for Vec<T> {
	fn get_definition() -> Definition { Definition::array(definition_of::<T>()) }
}

impl<T: FromJson> FromJson for Vec<T> {
	fn try_from_json(json: &Value) -> Result<Self> {
		let Value::Array(vec) = json
		else { bail!("Expected JSON value to be of type \"array\".\nGot: {json:?}") };

		vec.iter()
			.map(|val| T::try_from_json(val))
			.try_collect()
	}
}

impl<T: GetDefinition + ArrayElement> GetDefinition for Array<T> {
	fn get_definition() -> Definition { definition_of::<Vec<T>>() }
}

impl<T: FromJson + ArrayElement> FromJson for Array<T> {
	fn try_from_json(json: &Value) -> Result<Self> {
		let Value::Array(vec) = json
		else { bail!("Expected JSON value to be of type \"array\".\nGot: {json:?}") };

		vec.iter()
			.map(|val| T::try_from_json(val))
			.try_collect()
	}
}

impl<T: GetDefinition> GetDefinition for HashSet<T> {
	fn get_definition() -> Definition { definition_of::<Vec<T>>() }
}

impl<T: FromJson + Eq + Hash> FromJson for HashSet<T> {
	fn try_from_json(json: &Value) -> Result<Self> {
		let Value::Array(vec) = json
		else { bail!("Expected JSON value to be of type \"array\".\nGot: {json:?}") };

		vec.iter()
			.map(|val| T::try_from_json(val))
			.try_collect()
	}
}

impl<T: GetDefinition> GetDefinition for BTreeSet<T> {
	fn get_definition() -> Definition { definition_of::<Vec<T>>() }
}

impl<T: FromJson + Ord> FromJson for BTreeSet<T> {
	fn try_from_json(json: &Value) -> Result<Self> {
		let Value::Array(vec) = json
		else { bail!("Expected JSON value to be of type \"array\".\nGot: {json:?}") };

		vec.iter()
			.map(|val| T::try_from_json(val))
			.try_collect()
	}
}

packed_array_definitions!(
	PackedByteArray: [u8]
	PackedInt32Array: [i32]
	PackedInt64Array: [i64]
	PackedFloat32Array: [f32]
	PackedFloat64Array: [f64]
	PackedStringArray: [GString]
	PackedVector2Array: [Vector2]
	PackedVector3Array: [Vector3]
	PackedColorArray: [Color]
	PackedVector4Array: [Vector4]
);

tuple_definitions!(T1);
tuple_definitions!(T1, T2);
tuple_definitions!(T1, T2, T3);
tuple_definitions!(T1, T2, T3, T4);
tuple_definitions!(T1, T2, T3, T4, T5);
tuple_definitions!(T1, T2, T3, T4, T5, T6);
tuple_definitions!(T1, T2, T3, T4, T5, T6, T7);
tuple_definitions!(T1, T2, T3, T4, T5, T6, T7, T8);

fn try_value_at_key<T: FromJson>(key: &str, properties: &Map<String, Value>) -> Result<T> {
	let value = properties
		.get(key)
		.ok_or_else(|| anyhow!("Expected property `{key}` to be present."))?;

	T::try_from_json(value)
}