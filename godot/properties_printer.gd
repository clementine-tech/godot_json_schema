extends Node


var gen_array: Array[Dictionary]


func _ready():
	var script: Script = load("res://properties_printer.gd")
	for prop in script.get_script_property_list():
		print(prop)
