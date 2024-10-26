class_name Person


enum Gender {
	Male,
	Female,
}


var gender: Gender
var first_name: String
var last_name: String
var password: String
var facts: Array[Fact]


func properties_string() -> String:
	var facts_str = ""
	for fact in facts:
		facts_str += "\tText: %s \n\tSalientWord: %s, IsPasswordRelated: %s \n\n" % [fact.text, fact.salient_word, fact.is_password_related]

	return "Name: %s %s, Gender: %s, Password: %s\nFacts:\n%s" % [first_name, last_name, gender, password, facts_str]
