extends Node


func _ready():
	run_tests()


func run_tests():
	test_other_types()
	
	# Run this first since the second is a coroutine, otherwise the logs will mix.
	print("Testing person class schema round trip")
	test_person()
	
	print("\n\n")
	
	print("Testing structured output call to OpenAI")
	await test_structured_output()
	
	print("Testing structured output call to OpenAI with 3 people")
	await test_structured_3_people()


# - Vector2 => { type: VariantType::VECTOR2 }
# - String => { type: VariantType::STRING }
# - Enum `Gender` => { type: VariantType::INT, class_name: &"Person.Gender", usage: PropertyUsageFlags::CLASS_IS_ENUM }
# - Class `Fact` => { type: VariantType::OBJECT, class_name: "Fact" }
# - UntypedArray => { type: VariantType::ARRAY }
# - Dictionary => { type: VariantType::DICTIONARY }
# - Array<int> => { type: VariantType::ARRAY, hint: PropertyHint::ARRAY_TYPE, hint_string: "int" } 
# - Array<Dictionary> => { type: VariantType::ARRAY, hint: PropertyHint::ARRAY_TYPE, hint_string: "Dictionary"  }
# - Array<`Gender`> => { type: VariantType::ARRAY, hint: PropertyHint::ARRAY_TYPE, hint_string: "Person.Gender" }
# - Array<`Fact`> => { type: VariantType::ARRAY, hint: PropertyHint::ARRAY_TYPE, hint_string: "Fact" }
func test_other_types():
	test_type_info(TYPE_VECTOR2)
	test_type_info(TYPE_STRING)
	test_type_info(TYPE_INT, &"Person.Gender", PROPERTY_HINT_NONE, "", PROPERTY_USAGE_CLASS_IS_ENUM)
	test_type_info(TYPE_OBJECT, &"Fact")
	test_type_info(TYPE_ARRAY)
	test_type_info(TYPE_DICTIONARY)
	test_type_info(TYPE_ARRAY, &"", PROPERTY_HINT_ARRAY_TYPE, "int")
	test_type_info(TYPE_ARRAY, &"", PROPERTY_HINT_ARRAY_TYPE, "Dictionary")
	test_type_info(TYPE_ARRAY, &"", PROPERTY_HINT_ARRAY_TYPE, "Person.Gender")
	test_type_info(TYPE_ARRAY, &"", PROPERTY_HINT_ARRAY_TYPE, "Fact")


func test_type_info(
	variant_type: Variant.Type, 
	_class_name: StringName = "", 
	hint: PropertyHint = PROPERTY_HINT_NONE, 
	hint_string: String = "", 
	usage: PropertyUsageFlags = PROPERTY_USAGE_NONE,
):
	var result = GodotSchema.from_type_info(variant_type, _class_name, hint, hint_string, usage)
	if result is String:
		printerr(result)


func test_person():
	var json = """
	{
		"gender": "Female",
		"first_name":"Charlie",
		"last_name":"Whimsby",
		"facts":[
			{
				"text":"Charlie once won an award for the largest carrot grown in the annual town vegetable contest.",
				"salient_word":"carrot",
				"is_password_related":true
			},{
				"text":"Despite being allergic, Charlie has a pet cat named Whiskers.",
				"salient_word":"Whiskers",
				"is_password_related":false
			},{
				"text":"Charlie's favorite pastime is competing in local hot air balloon races.",
				"salient_word":"balloon",
				"is_password_related":true
			},{
				"text":"They absolutely despise the taste and smell of cucumbers.",
				"salient_word":"cucumbers",
				"is_password_related":false
			},{
				"text":"Charlie met their best friend, Amber, during a volleyball match in college.",
				"salient_word":"Amber",
				"is_password_related":true
			}
		],
		"password":"carrotballoonAmber",
		"main_fact": {
			"text": "A lonely fact that must never be grouped with the other silly ones.",
			"salient_word":"mix",
			"is_password_related":true
		}
	}
	"""
	
	var schema_res = GodotSchema.from_class_name(&"Person")
	if schema_res is String:
		printerr(schema_res)
		return
	
	var schema: GodotSchema = schema_res
	print("Schema:\n" + schema.json)
	
	var result = schema.instantiate(json)
	
	if result is Person:
		print("Instantiated Person:\n" + result.properties_string())
	else:
		printerr("Instantiation failed. Error: " + str(result))


func test_structured_output():
	var lib = SchemaLibrary.new()
	var schema_res = lib.generate_named_class_schema(&"Person")
	if schema_res is String:
		printerr(schema_res)
		return
	
	var person_schema: GodotSchema = schema_res
	print("Schema:\n" + person_schema.json)
	
	var response_format = person_schema.open_ai_response_format("Person")
	print("Response format:\n" + str(response_format))

	var llm_client = LLMClientNode.create(
		"https://clm-proxy.deno.dev/v1",
		"your_api_key_here"
	)
	add_child(llm_client)

	var model = "gpt-4o"
	var provider = "OPENAI"

	var messages: Array = [
		{"role": "system", "content": "You are a helpful assistant. Answer in JSON format."},
		{"role": "user", "content": "Generate a random person with a first name, last name, gender, and a list of facts about them. Use some of the facts to generate a password."}
	]

	var chat_result
	# JSON object
	# chat_result = llm_client.chat_completion_structured(messages, model, provider, 'json_object')

	# Or, you can do JSON schema
	print("Schema string:\n" + str(response_format))
	chat_result = llm_client.chat_completion_structured(messages, model, provider, response_format)
	
	var chat_output = await chat_result.finished
	print("Chat completion result:")
	print(chat_output)

	# Instantiate the class
	var result = person_schema.instantiate(chat_output)
	if result is Person:
		print("Instantiated Person:\n" + result.properties_string())
	else:
		printerr("Instantiation failed. Error: " + str(result))


func test_structured_3_people():
	var schema_res = GodotSchema.from_class_name(&"Person")
	if schema_res is String:
		printerr(schema_res)
		return
	
	var array_schema_res = schema_res.get_array_schema("Person")
	if array_schema_res is String:
		printerr(array_schema_res)
		return
	
	var array_schema: GodotSchema = array_schema_res
	
	var response_format = array_schema.open_ai_response_format("PersonTrio")
	print("Response format:\n" + str(response_format))
	
	var llm_client = LLMClientNode.create(
		"https://clm-proxy.deno.dev/v1",
		"your_api_key_here"
	)
	add_child(llm_client)

	var model = "gpt-4o"
	var provider = "OPENAI"

	var messages: Array = [
		{"role": "system", "content": "You are a helpful assistant. Answer in JSON format."},
		{"role": "user", "content": "Generate 3 random people, each with a first name, last name, gender, and a list of facts about them. Use some of the facts to generate a password for each person."}
	]
	
	var chat_result = llm_client.chat_completion_structured(messages, model, provider, response_format)
	
	var chat_output = await chat_result.finished
	print("Chat completion result:\n" + str(chat_output))
	
	var result = array_schema.instantiate(chat_output)
	if result is Array[Person]:
		print("Instantiated people:\n")
		for person in result:
			print(person.properties_string())
	else:
		printerr("Instantiation failed. Error: " + str(result))
