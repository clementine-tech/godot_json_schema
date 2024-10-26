# godot-json-schema
Convert Godot classes into JSON Schemas and instantiate Godot objects from JSON objects.

 Generate JSON schemas from Godot classes, and instantiate Godot classes from json.


## Running the integration tests

First, build the project with the "integration_tests" feature enabled:
```
cargo build -F integration_tests
```

Then, you can run the tests by playing the main scene "godot/schema_tester.tscn".

## Setup Example (GDScript)

Consider the given class:
 ```js
 class_name Person

 var name: String
 var age: int
 ```

### Step 1
Create a `SchemaLibrary` instance, which will be used to generate and cache your schemas:
 ```js
 var library = SchemaLibrary.new()
 ```

### Step 2
Generate a schema for your class.

 Note 1: If the class is "user-made" (i.e. not part of the engine),
 the generated schema will only contain the properties of the class and any other GDScript classes that it inherits from. 
 This means that properties from any ancestor engine classes that you inherit from will not be included in the schema.

 Note 2: If a property of your class is a Godot class, you do not have to generate a schema for the property's type,
 this crate will automatically generate all the necessary "dependency" schemas for you.
 These dependencies are included as definitions in the schema (i.e. included in the "$defs" dictionary).

 A schema can be generated in two different ways:

 ### Method 1 - "`generate_named_class_schema(ClassName)`"
 Requires your class to be registered in `ProjectSettings::get_global_class_list()`.

 For a class to be registered, it needs to contain a "`class_name MyName`" statement at the top of the script.

 "`generate_named_class_schema()`" will return "`Nil`"(if it succeeded) or "`String`" as an error message.

 ```js
 var result = library.generate_named_class_schema(&"Person")

 if result is String:
     print_err(result)
 ```

 ### Method 2 - "`generate_unnamed_class_schema(Script)`"
 Does not require your class to be registered in `ProjectSettings::get_global_class_list()`.

 "`generate_unnamed_class_schema`" will return "`Nil`"(if it succeeded) or "`String`" as an error message.

 ```js
 var script = preload("res://person.gd")
 var result = library.generate_unnamed_class_schema(script)

 if result is String:
     print_err(result)
 ```

 ## Instantiating Godot classes from JSON

 After setting up your library and ensuring that your necessary schemas are generated,
 you can instantiate a given class from JSON input containing the properties of the class.

 Notes:
 - The JSON input must be valid according to the schema.
 - The JSON input must contain all properties defined in the schema (i.e. the schema's "required" array has all of your class's properties).
 - The JSON input must not contain any additional properties (i.e. the schema's "additionalProperties" key is set to false).

 ```js
 var person_properties_json = 
     """
     {
         "name": "John Doe",
         "age": 43
     }
     """

 var result = library.instantiate_named_class(&"Person", person_properties_json)

 if result is String:
     print_err(result)
 else:
     var person: Person = result
     assert(person.name == "John Doe")
     assert(person.age == 43)
 ```
 
 Note: If the class does not have a global name (i.e. it has a "class_name MyName" statement at the top of the script),
 use "`instantiate_unnamed_class(Script, PropertiesJson)`" instead.

 ## Accessing the json representation of a generated schema

 Use either "`get_named_class_schema(ClassName)`" or "`get_unnamed_class_schema(Script)`" on your `SchemaLibrary` instance.

 ```js
 var json: String = library.get_named_class_schema(&"Person")

 if json.is_empty():
     print_err("No schema found for class Person.")
 else:
     print("Generated schema:\n" + json)
 ```

## Limitations
1. Properties of your root schema cannot be unnamed Godot classes. They must have a "class_name MyName" statement at the top of the script.
   Note that this is only imposed on the property types, not the root schema's class.

2. Serializing/deserializing Godot's built-in global enums (any enum in [global scope](https://docs.godotengine.org/en/stable/classes/class_%40globalscope.html#enumerations)) is not supported.