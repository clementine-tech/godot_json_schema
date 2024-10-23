extends Node


func _ready():
	test_person()


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
