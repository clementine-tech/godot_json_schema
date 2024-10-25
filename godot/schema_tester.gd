extends Node


func _ready():
	print("Testing structured output call to OpenAI")
	test_structured_output()
	# print("Testing person class schema round trip")
	# test_person()


func test_person():
	var lib = SchemaLibrary.new()
	
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
	
	var res = lib.generate_named_class_schema(&"Person")
	if res is String:
		printerr(res)
	
	var schema = lib.get_named_class_schema(&"Person")
	print("Schema:\n" + str(schema))
	
	var result = lib.instantiate_named_class(&"Person", json)
	
	if result is Person:
		var facts = ""
		for fact in result.facts:
			facts += "\tText: %s \n\tSalientWord: %s, IsPasswordRelated: %s \n\n" % [fact.text, fact.salient_word, fact.is_password_related]
		
		print("Name: %s %s, Gender: %s, Password: %s\nFacts:\n%s" % [result.first_name, result.last_name, result.gender, result.password, facts])
	else:
		print("Instantiation failed. Error: " + str(result))


func test_structured_output():
	print("Testing structured output")

	var lib = SchemaLibrary.new()
	var res = lib.generate_named_class_schema(&"Person")
	if res is String:
		printerr(res)
	
	var person_schema = lib.get_named_class_schema(&"Person")
	print("Schema:\n" + str(person_schema))
	
	var response_format = lib.construct_response_format(person_schema, "Person")
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
	print("Schema string: " + str(response_format))
	chat_result = llm_client.chat_completion_structured(messages, model, provider, response_format)
	
	

	var chat_output = await chat_result.finished
	print("Chat completion result:")
	print(chat_output)

	# Instantiate the class
	var result = lib.instantiate_named_class(&"Person", chat_output)
	if result is Person:
		print("Instantiated person: " + result.first_name + " " + result.last_name)
	else:
		print("Instantiation failed. Error: " + str(result))

func gd_construct_response_schema(schema: String ,className: String) -> String:	
	var responseFormat = "{ \"type\": \"json_schema\"," + \
						" \"json_schema\": { " + \
						" \"name\": \"" + className + "\"," + \
						" \"schema\": " + schema + " } }"
	return responseFormat
