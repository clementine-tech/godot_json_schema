extends Node


func _ready():
	# Run this first since the second is a coroutine, otherwise the logs will mix.
	print("Testing person class schema round trip")
	test_person()
	
	print("\n\n")
	
	print("Testing structured output call to OpenAI")
	test_structured_output()


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
		"password":"carrotballoonAmber"
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
	print("Testing structured output")

	var lib = SchemaLibrary.new()
	var schema_res = lib.generate_named_class_schema(&"Person")
	if schema_res is String:
		printerr(schema_res)
		return
	
	var person_schema: GodotSchema = schema_res
	print("Schema:\n" + person_schema.json)
	
	var response_format = person_schema.open_ai_response_format()
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
